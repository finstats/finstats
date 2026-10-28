//! The one destination that is not an HTTP request: a letter, over SMTP.
//!
//! Everything else FinStats sends is a POST of JSON, and `channels.rs` is all of it. Mail is different
//! enough to keep apart: a different transport, a different library, an envelope with a sender in it, and
//! a password that must never cross the wire in the clear. Three rules hold here:
//!
//! - **It is encrypted, always.** `smtps://` is TLS from the first byte (465); `smtp://` starts plain and
//!   must upgrade with STARTTLS (587) or the letter does not go. A relay with a certificate of its own
//!   making is covered by the same administrator-only "accept invalid certificates" switch every other
//!   connection has: a switch, not the default.
//! - **The address is the destination's, not FinStats'.** The mailbox to send to is stored like an ntfy
//!   topic; the sender is asked for because most servers refuse a `From` they do not know.
//! - **Nothing that comes back is repeated.** Errors say a host and a kind of failure, exactly like
//!   `channels::explain` does, and never the server's own words or the password it refused.

use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::channels::SendError;
use crate::i18n::t;
use crate::notify::Message;

/// How long one exchange with a mail server may take. A mail server is slower than a webhook, and a destination that
/// is waiting on one holds up its own letters (`notify::MAIL_TIMEOUT` is the whole of one).
const TIMEOUT_S: u64 = 10;
/// A subject is one line in a header; a body may be long, but not unbounded.
const SUBJECT_MAX: usize = 200;
const BODY_MAX: usize = 8_000;

/// Where to connect, and how it is encrypted. `starttls` = start in the clear and upgrade, which is
/// required rather than attempted: FinStats does not fall back to sending a password in the open.
#[derive(Debug, PartialEq, Eq)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub starttls: bool,
}

/// The stored address, read. `smtps://` is 465 and encrypted from the first byte, `smtp://` is 587 and
/// must upgrade; a port that was typed always wins.
pub fn server(url: &str) -> Result<Server> {
    let parsed = reqwest::Url::parse(url).map_err(|_| anyhow!(t("server.mail.bad_url", &[])))?;
    let starttls = match parsed.scheme() {
        "smtps" => false,
        "smtp" => true,
        _ => bail!(t("server.mail.scheme", &[])),
    };
    let host = parsed.host_str().filter(|h| !h.is_empty()).ok_or_else(|| anyhow!(t("server.mail.needs_host", &[])))?;
    Ok(Server { host: host.to_string(), port: parsed.port().unwrap_or(if starttls { 587 } else { 465 }), starttls })
}

/// One letter, before anything is connected to. Pure, so what is actually sent has a test under it.
pub struct Letter {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub text: String,
}

/// A header is one line, so nothing that could begin a second one survives into the subject: a title
/// comes from a library, and a library is full of other people's spellings.
fn one_line(text: &str) -> String {
    let flat: String = text.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    flat.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(SUBJECT_MAX).collect()
}

/// What FinStats accepts as a mailbox: something lettre will send to, whose domain could be looked up.
/// Nothing here is a guess about whether it exists, only that it is an address at all.
pub fn address(input: &str) -> Result<String> {
    let text = input.trim();
    let parsed: Address = text.parse().map_err(|_| anyhow!(t("server.mail.not_an_address", &[("address", text.into())])))?;
    if !parsed.domain().contains('.') {
        bail!(t("server.mail.no_domain", &[("address", text.into())]));
    }
    Ok(text.to_string())
}

fn mailbox(address: &str, name: Option<&str>) -> Result<Mailbox> {
    let address: Address = address.trim().parse().map_err(|_| anyhow!(t("server.mail.not_an_address", &[("address", address.trim().into())])))?;
    Ok(Mailbox::new(name.map(str::to_string), address))
}

pub fn letter(from: &str, to: &str, m: &Message) -> Result<Letter> {
    mailbox(from, None)?;
    mailbox(to, None)?;
    let text: String = m.text().chars().take(BODY_MAX).collect();
    Ok(Letter { from: from.trim().to_string(), to: to.trim().to_string(), subject: one_line(&m.title), text })
}

/// Everything a destination stored, as sending one letter needs it.
pub struct Account<'a> {
    pub url: &'a str,
    pub from: &'a str,
    pub to: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    /// The administrator-only switch: a relay with a certificate of its own making.
    pub lax: bool,
    /// For a personal destination, the address its name was checked at, connected to instead of a second lookup.
    /// The certificate is still checked against the name.
    pub connect_to: Option<std::net::IpAddr>,
}

/// Send one letter. Like every other destination, what comes back from here names a host and a kind of
/// failure and never the server's own words.
pub async fn deliver(acct: Account<'_>, m: &Message) -> std::result::Result<(), SendError> {
    let refuse = |e: anyhow::Error| SendError::of(format!("{e}"));
    let server = server(acct.url).map_err(refuse)?;
    let letter = letter(acct.from, acct.to, m).map_err(refuse)?;
    let built = lettre::Message::builder()
        .from(mailbox(&letter.from, Some("FinStats")).map_err(refuse)?)
        .to(mailbox(&letter.to, None).map_err(refuse)?)
        .subject(letter.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(letter.text)
        .map_err(|_| SendError::of(t("server.mail.not_a_letter", &[])))?;

    let tls = TlsParameters::builder(server.host.clone())
        .dangerous_accept_invalid_certs(acct.lax)
        .dangerous_accept_invalid_hostnames(acct.lax)
        .build()
        .map_err(|_| SendError::of(t("server.mail.no_tls_setup", &[("host", server.host.as_str().into())])))?;
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(acct.connect_to.map_or_else(|| server.host.clone(), |ip| ip.to_string()))
        .port(server.port)
        .timeout(Some(Duration::from_secs(TIMEOUT_S)))
        .tls(if server.starttls { Tls::Required(tls) } else { Tls::Wrapper(tls) });
    if !acct.username.is_empty() {
        builder = builder.credentials(Credentials::new(acct.username.to_string(), acct.password.to_string()));
    }
    builder.build().send(built).await.map(|_| ()).map_err(|e| SendError { timed_out: e.is_timeout(), ..SendError::of(explain(&e, &server.host)) })
}

/// Why a letter did not go, in words, and without anything the server said back.
fn explain(e: &lettre::transport::smtp::Error, host: &str) -> String {
    let key = if e.is_timeout() {
        "timeout"
    } else if e.is_tls() {
        "tls"
    } else {
        match e.status().map(|s| s.severity) {
            Some(lettre::transport::smtp::response::Severity::PermanentNegativeCompletion) => "refused",
            Some(_) => "busy",
            None => "connect",
        }
    };
    t(&format!("server.mail.{key}"), &[("host", host.into())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notify::{INFO, Message};

    /// A personal destination's letter goes to the address that was checked, never to a second lookup of its name.
    #[tokio::test]
    async fn a_letter_goes_to_the_address_that_was_checked() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("smtp://mail.invalid:{port}");
        let acct = Account { url: &url, from: "finstats@example.com", to: "alice@example.com", username: "", password: "", lax: false, connect_to: Some("127.0.0.1".parse().unwrap()) };
        let m = msg();
        tokio::select! {
            r = deliver(acct, &m) => panic!("gave up without connecting to the address given: {:?}", r.err().map(|e| e.message)),
            r = listener.accept() => assert!(r.is_ok()),
            _ = tokio::time::sleep(Duration::from_secs(5)) => panic!("nothing connected to the address given"),
        }
    }

    fn msg() -> Message {
        Message {
            kind: crate::notify::Kind::NewItems,
            at: 1_000,
            user: None,
            title: "New in the library: Big Buck Bunny".into(),
            body: "One film was added.".into(),
            link: Some("https://finstats.example/libraries".into()),
            severity: INFO.into(),
            fields: vec![("Library".into(), "Films".into())],
        }
    }

    #[test]
    fn an_address_says_where_to_connect_and_how_it_is_encrypted() {
        let s = server("smtps://smtp.example.com").unwrap();
        assert_eq!((s.host.as_str(), s.port, s.starttls), ("smtp.example.com", 465, false), "TLS from the first byte");
        let s = server("smtp://smtp.example.com").unwrap();
        assert_eq!((s.host.as_str(), s.port, s.starttls), ("smtp.example.com", 587, true), "plain, and it must upgrade");
        let s = server("smtp://mail.example.com:2525").unwrap();
        assert_eq!((s.host.as_str(), s.port), ("mail.example.com", 2525), "a port that was typed is the port");
        assert!(server("https://smtp.example.com").is_err(), "an address that is not SMTP is not a mail server");
        assert!(server("smtps://").is_err());
    }

    #[test]
    fn a_letter_carries_the_whole_message_and_says_who_it_is_from() {
        let l = letter("finstats@example.com", "me@example.com", &msg()).unwrap();
        assert_eq!(l.subject, "New in the library: Big Buck Bunny", "the subject is the title, with nothing bolted on");
        assert_eq!((l.from.as_str(), l.to.as_str()), ("finstats@example.com", "me@example.com"));
        assert!(l.text.contains("One film was added.") && l.text.contains("Library: Films"), "a mailbox gets every field: {}", l.text);
        assert!(l.text.contains("https://finstats.example/libraries"));
    }

    #[test]
    fn an_envelope_that_is_not_one_is_refused_rather_than_sent_anyway() {
        assert!(letter("not an address", "me@example.com", &msg()).is_err());
        assert!(letter("finstats@example.com", "me at example.com", &msg()).is_err());
        // A header is one line: anything that could start a second one is not a subject.
        assert!(address("me@example").is_err(), "a mailbox needs a domain that resolves");
        assert_eq!(address("  me@example.com ").unwrap(), "me@example.com");
        let sneaky = Message { title: "New\r\nBcc: someone@example.com".into(), ..msg() };
        let l = letter("finstats@example.com", "me@example.com", &sneaky).unwrap();
        assert!(!l.subject.contains('\n') && !l.subject.contains('\r'), "a subject is one line: {:?}", l.subject);
    }
}

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

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::{Credentials, DEFAULT_MECHANISMS};
use lettre::transport::smtp::client::{AsyncSmtpConnection, AsyncTokioStream, TlsParameters};
use lettre::transport::smtp::extension::ClientId;
use lettre::Address;

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
    let host = server.host.as_str();
    let said = |key: &str| t(&format!("server.mail.{key}"), &[("host", host.into())]);
    let fail = |e: lettre::transport::smtp::Error| SendError { timed_out: e.is_timeout(), ..SendError::of(explain(&e, host)) };

    // lettre's own transport opens its own connection, and reads a reply until a line ends however long that takes: a
    // connection of ours, read no further than `READ_MAX`, is handed to it instead.
    let at = acct.connect_to.map_or_else(|| server.host.clone(), |ip| ip.to_string());
    let tcp = match tokio::time::timeout(Duration::from_secs(TIMEOUT_S), tokio::net::TcpStream::connect((at.as_str(), server.port))).await {
        Err(_) => return Err(SendError { timed_out: true, ..SendError::of(said("timeout")) }),
        Ok(Err(_)) => return Err(SendError::of(said("connect"))),
        Ok(Ok(tcp)) => tcp,
    };
    let capped = Capped { io: tcp, left: READ_MAX, idle: None };
    let hello = ClientId::default();
    let stream: Box<dyn AsyncTokioStream> = if server.starttls {
        Box::new(capped)
    } else {
        #[allow(deprecated)]
        let mut wrapped = lettre::transport::smtp::client::AsyncNetworkStream::use_existing_tokio1(Box::new(capped));
        wrapped.upgrade_tls(tls.clone()).await.map_err(fail)?;
        Box::new(Encrypted(wrapped))
    };
    let mut conn = AsyncSmtpConnection::connect_with_transport(stream, &hello).await.map_err(fail)?;
    if server.starttls {
        conn.starttls(tls, &hello).await.map_err(fail)?;
    }
    if !acct.username.is_empty() {
        conn.auth(DEFAULT_MECHANISMS, &Credentials::new(acct.username.to_string(), acct.password.to_string())).await.map_err(fail)?;
    }
    conn.send(built.envelope(), &built.formatted()).await.map_err(fail)?;
    let _ = conn.quit().await;
    Ok(())
}

/// The most a mail server may send over one letter, its certificates included. A real exchange is a few kilobytes.
const READ_MAX: u64 = 1 << 20;

/// The connection to a mail server: read no further than `left` bytes, and no read waits longer than `TIMEOUT_S`.
struct Capped {
    io: tokio::net::TcpStream,
    left: u64,
    idle: Option<std::pin::Pin<Box<tokio::time::Sleep>>>,
}

impl std::fmt::Debug for Capped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Capped")
    }
}

impl AsyncTokioStream for Capped {
    fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.io.peer_addr()
    }
}

impl tokio::io::AsyncRead for Capped {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        if self.left == 0 {
            return Poll::Ready(Err(std::io::Error::other("the mail server sent more than any reply holds")));
        }
        let before = buf.filled().len();
        match Pin::new(&mut self.io).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                let got = (buf.filled().len() - before) as u64;
                self.left = self.left.saturating_sub(got);
                self.idle = None;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => {
                let idle = self.idle.get_or_insert_with(|| Box::pin(tokio::time::sleep(Duration::from_secs(TIMEOUT_S))));
                match idle.as_mut().poll(cx) {
                    Poll::Ready(()) => Poll::Ready(Err(std::io::ErrorKind::TimedOut.into())),
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

impl tokio::io::AsyncWrite for Capped {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.io).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.io).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.io).poll_shutdown(cx)
    }
}

/// `smtps://`: the capped connection inside TLS, as lettre's own stream type, which speaks `futures-io`, turned back into
/// the tokio stream `connect_with_transport` takes.
#[allow(deprecated)]
#[derive(Debug)]
struct Encrypted(lettre::transport::smtp::client::AsyncNetworkStream);

impl AsyncTokioStream for Encrypted {
    fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.0.peer_addr()
    }
}

impl tokio::io::AsyncRead for Encrypted {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        match futures_io::AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf.initialize_unfilled()) {
            Poll::Ready(Ok(n)) => {
                buf.advance(n);
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl tokio::io::AsyncWrite for Encrypted {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        futures_io::AsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        futures_io::AsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        futures_io::AsyncWrite::poll_close(Pin::new(&mut self.0), cx)
    }
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

    /// A server that answers `220 ` and then nothing but `A`, never a line's end, is let go once FinStats has read more
    /// than any real reply holds. lettre grows one line until it sees a newline, so such a server once took FinStats to
    /// 1.6 GB in two seconds.
    #[tokio::test]
    async fn a_reply_that_never_ends_a_line_is_read_no_further_than_any_reply_could_be() {
        use tokio::io::AsyncWriteExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let sent = tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let chunk = vec![b'A'; 64 << 10];
            let mut sent = 0usize;
            if s.write_all(b"220 ").await.is_err() {
                return sent;
            }
            while sent < 200 << 20 {
                if s.write_all(&chunk).await.is_err() {
                    return sent;
                }
                sent += chunk.len();
            }
            std::future::pending::<()>().await;
            sent
        });
        let url = format!("smtp://mail.invalid:{port}");
        let acct = Account { url: &url, from: "finstats@example.com", to: "alice@example.com", username: "", password: "", lax: false, connect_to: Some("127.0.0.1".parse().unwrap()) };
        let m = msg();
        let r = tokio::time::timeout(Duration::from_secs(5), deliver(acct, &m)).await.expect("still reading the endless reply after 5 s");
        assert!(r.is_err(), "an endless reply was taken for an answer");
        let sent = tokio::time::timeout(Duration::from_secs(5), sent).await.expect("the server was never let go").unwrap();
        assert!(sent < 8 << 20, "{sent} bytes were read of one reply");
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

//! The nine ways a message leaves FinStats, and nothing else about notifications.
//!
//! A plain **webhook** (JSON, the shape documented in the HTTP API page of github.com/finstats/docs), **Discord** (an embed),
//! **ntfy** and **Gotify**. Two rules hold for all of them:
//!
//! - **The address is a secret.** A Discord webhook URL *is* its credential, and an ntfy topic is
//!   whoever knows it, so a URL is never logged, never put in an error and never sent to a browser.
//!   Errors say a status and a kind of failure, the way `services.rs` does, and nothing the other side
//!   returned.
//! - **No redirect is followed.** These requests carry tokens in headers and in the URL itself;
//!   `services::Http` already refuses redirects, so it is the client used here too.
//!
//! Both ntfy and Gotify are published to in their JSON form rather than through headers, because a title
//! is a film title: "Amélie" in an HTTP header is not something to find out about in the field.

use anyhow::Result;
use reqwest::StatusCode;
use reqwest::header::RETRY_AFTER;
use serde_json::{Value, json};

use crate::i18n::t;
use crate::notify::{ALERT, Message, Target, WARN};
use crate::state::App;

/// Discord's own limits, and sensible ones for the rest.
const TITLE_MAX: usize = 240;
const BODY_MAX: usize = 3_500;
const FIELD_MAX: usize = 900;
const FIELDS_MAX: usize = 20;

/// One field a channel needs beyond an address, a token and the one beside them. Only email has any:
/// a letter needs a sender, and often a user name that is not the sender.
pub struct Extra {
    pub key: &'static str,
    /// This one holds an e-mail address, and is checked as one before it is stored.
    pub address: bool,
    pub example: &'static str,
    pub required: bool,
}

impl Extra {
    /// What the field is called (`server.channels.extra.<key>.label`), in the language of whoever is reading.
    pub fn title(&self) -> String {
        t(&format!("server.channels.extra.{}.label", self.key), &[])
    }

    /// One line on what goes in it, in the language of whoever is reading.
    pub fn help_text(&self) -> String {
        t(&format!("server.channels.extra.{}.help", self.key), &[])
    }
}

const EMAIL_EXTRAS: [Extra; 2] = [
    Extra { key: "from", address: true, example: "finstats@example.com", required: true },
    Extra { key: "username", address: false, example: "finstats@example.com", required: false },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Channel {
    Discord,
    Telegram,
    Slack,
    Email,
    Ntfy,
    Gotify,
    Pushover,
    Pushbullet,
    Webhook,
}

impl Channel {
    pub const ALL: [Channel; 9] = [
        Channel::Discord, Channel::Telegram, Channel::Slack, Channel::Email, Channel::Ntfy,
        Channel::Gotify, Channel::Pushover, Channel::Pushbullet, Channel::Webhook,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Channel::Webhook => "webhook",
            Channel::Discord => "discord",
            Channel::Slack => "slack",
            Channel::Telegram => "telegram",
            Channel::Email => "email",
            Channel::Ntfy => "ntfy",
            Channel::Gotify => "gotify",
            Channel::Pushover => "pushover",
            Channel::Pushbullet => "pushbullet",
        }
    }

    pub fn from_key(key: &str) -> Option<Channel> {
        Channel::ALL.into_iter().find(|c| c.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            Channel::Webhook => "Webhook",
            Channel::Discord => "Discord",
            Channel::Slack => "Slack",
            Channel::Telegram => "Telegram",
            Channel::Email => "Email",
            Channel::Ntfy => "ntfy",
            Channel::Gotify => "Gotify",
            Channel::Pushover => "Pushover",
            Channel::Pushbullet => "Pushbullet",
        }
    }

    /// What a page calls it, in the reader's language: `label` is the name itself, which only "Webhook" and "Email"
    /// are not.
    pub fn title(self) -> String {
        match self {
            Channel::Webhook | Channel::Email => t(&format!("server.channels.{}", self.key()), &[]),
            _ => self.label().to_string(),
        }
    }

    /// One line on what it is, in the reader's language.
    pub fn what(self) -> String {
        t(&format!("server.channels.what.{}", self.key()), &[])
    }

    /// Where the service lives, when that is not something anybody should have to type. A channel with
    /// one of these asks only for its token: the address is the service's own and never changes.
    pub fn fixed_url(self) -> Option<&'static str> {
        match self {
            Channel::Telegram => Some("https://api.telegram.org"),
            Channel::Pushover => Some("https://api.pushover.net"),
            Channel::Pushbullet => Some("https://api.pushbullet.com"),
            _ => None,
        }
    }

    /// The address to show in the field, and empty for a channel that fixes its own.
    pub fn example(self) -> &'static str {
        match self {
            Channel::Webhook => "https://example.com/hooks/finstats",
            Channel::Discord => "https://discord.com/api/webhooks/123456789/xxxxxxxx",
            Channel::Slack => "https://hooks.slack.com/services/T00000000/B00000000/xxxxxxxx",
            Channel::Email => "smtps://smtp.example.com:465",
            Channel::Ntfy => "https://ntfy.sh",
            Channel::Gotify => "http://192.168.1.10:8070",
            Channel::Telegram | Channel::Pushover | Channel::Pushbullet => "",
        }
    }

    /// The one field beside the address and the token, when a channel has one: which topic, which chat,
    /// whose key, which mailbox. `None` for a channel that says all of that in the address itself.
    pub fn topic_label(self) -> Option<&'static str> {
        match self {
            Channel::Ntfy => Some("Topic"),
            Channel::Telegram => Some("Chat ID"),
            Channel::Pushover => Some("User or group key"),
            Channel::Email => Some("To"),
            _ => None,
        }
    }

    /// `topic_label`, in the reader's language.
    pub fn topic_title(self) -> Option<String> {
        self.needs_topic().then(|| t(&format!("server.channels.topic.{}.label", self.key()), &[]))
    }

    pub fn topic_help(self) -> String {
        if self.needs_topic() { t(&format!("server.channels.topic.{}.help", self.key()), &[]) } else { String::new() }
    }

    pub fn topic_example(self) -> &'static str {
        match self {
            Channel::Ntfy => "finstats-a8f3c1",
            Channel::Telegram => "-1001234567890",
            Channel::Pushover => "uQiRzpo4DXghDmr9QzzfQu27cmVRsG",
            Channel::Email => "me@example.com",
            _ => "",
        }
    }

    pub fn needs_topic(self) -> bool {
        self.topic_label().is_some()
    }

    /// What the secret beside the address is called, when there is one. A Discord or Slack webhook
    /// carries its own token in the URL, so there is nothing else to enter.
    pub fn secret_label(self) -> Option<String> {
        match self {
            Channel::Discord | Channel::Slack => None,
            _ => Some(t(&format!("server.channels.secret.{}", self.key()), &[])),
        }
    }

    pub fn secret_required(self) -> bool {
        matches!(self, Channel::Gotify | Channel::Telegram | Channel::Pushover | Channel::Pushbullet)
    }

    /// Anything else the channel cannot do without. Only email has any.
    pub fn extras(self) -> &'static [Extra] {
        match self {
            Channel::Email => &EMAIL_EXTRAS,
            _ => &[],
        }
    }

    /// Email is not an HTTP request at all: `mail.rs` has its own shape and its own transport.
    pub fn is_mail(self) -> bool {
        self == Channel::Email
    }
}

/// Why a message did not arrive. `retry_after` is what the other side asked for, in seconds.
pub struct SendError {
    pub message: String,
    pub retry_after: Option<i64>,
}

impl SendError {
    pub fn of(message: impl Into<String>) -> SendError {
        SendError { message: message.into(), retry_after: None }
    }
}

fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

fn fields_json(m: &Message, inline: bool) -> Vec<Value> {
    m.fields
        .iter()
        .take(FIELDS_MAX)
        .map(|(label, value)| json!({ "name": cut(label, 100), "value": cut(value, FIELD_MAX), "inline": inline }))
        .collect()
}

/// Discord's colour bar, and ntfy's and Gotify's priorities: the one place severity turns into a number.
fn colour(severity: &str) -> u32 {
    match severity {
        ALERT => 0xE5484D,
        WARN => 0xE0A300,
        _ => 0x4F8CFF,
    }
}

fn priority(severity: &str) -> i64 {
    match severity {
        ALERT => 5,
        WARN => 4,
        _ => 3,
    }
}

fn tags(severity: &str) -> Vec<&'static str> {
    match severity {
        ALERT => vec!["rotating_light"],
        WARN => vec!["warning"],
        _ => vec!["information_source"],
    }
}

/// Discord's colour bar as Slack wants it: the same number, written the way a web page would.
fn hex(severity: &str) -> String {
    format!("#{:06x}", colour(severity))
}

/// Pushover has its own scale, from -2 (never disturb) to 2 (emergency, which needs an acknowledgement
/// and is never what FinStats sends). Quiet for the everyday, the usual for a warning, loud for an alert.
fn pushover_priority(severity: &str) -> i64 {
    match severity {
        ALERT => 1,
        WARN => 0,
        _ => -1,
    }
}

/// How long a message may be on this channel. Pushover measures in its own, much smaller, units, so a
/// play that would fit anywhere else is cut to what it takes rather than refused by it.
fn body_max(channel: Channel) -> usize {
    match channel {
        Channel::Pushover => 1_000,
        Channel::Slack => 3_000,
        _ => BODY_MAX,
    }
}

/// What one destination is sent: where to, and the JSON body. Pure, so every shape has a test under it
/// and none of them needs a server to check. `None` for email, which is not an HTTP request at all:
/// `mail.rs` has its own shape and its own transport.
pub fn payload(channel: Channel, base_url: &str, topic: Option<&str>, secret: &str, m: &Message) -> Option<(String, Value)> {
    let title = cut(&m.title, TITLE_MAX);
    let body = cut(&m.body, body_max(channel));
    let text = cut(&m.text(), body_max(channel));
    let (kind_key, at, user) = (m.kind.key(), m.at, m.user.as_deref());
    Some(match channel {
        Channel::Email => return None,
        Channel::Webhook => (
            base_url.to_string(),
            json!({
                "event": kind_key, "severity": m.severity, "at": at, "title": title,
                "body": body, "link": m.link, "user": user,
                "fields": m.fields.iter().take(FIELDS_MAX).map(|(l, v)| json!({ "label": l, "value": cut(v, FIELD_MAX) })).collect::<Vec<_>>(),
                "source": concat!("finstats/", env!("CARGO_PKG_VERSION")),
            }),
        ),
        Channel::Discord => (
            base_url.to_string(),
            json!({
                "username": "FinStats",
                "embeds": [{
                    "title": title, "description": body, "url": m.link, "color": colour(&m.severity),
                    "timestamp": chrono::DateTime::from_timestamp(at, 0).map(|t| t.to_rfc3339()),
                    "fields": fields_json(m, true), "footer": { "text": "FinStats" },
                }],
            }),
        ),
        // Slack shows `text` in the list of conversations and the attachment when the message is opened,
        // so the title is said twice on purpose: the coloured card alone would be a blank line in the list.
        Channel::Slack => (
            base_url.to_string(),
            json!({
                "text": title,
                "attachments": [{
                    "color": hex(&m.severity), "title": title, "title_link": m.link, "text": body,
                    "fields": m.fields.iter().take(FIELDS_MAX).map(|(l, v)| json!({ "title": cut(l, 100), "value": cut(v, FIELD_MAX), "short": true })).collect::<Vec<_>>(),
                    "footer": "FinStats", "ts": at,
                }],
            }),
        ),
        // The bot token is the path, which is why nothing here is ever logged with its URL. No
        // `parse_mode`: a film title is not markup, and a title with a `_` in it would either break the
        // message or have to be escaped in a way that shows up in the text.
        Channel::Telegram => (
            format!("{base_url}/bot{secret}/sendMessage"),
            json!({
                "chat_id": topic.unwrap_or_default(),
                "text": format!("{title}\n{text}"),
                "disable_web_page_preview": true,
            }),
        ),
        Channel::Pushover => (
            format!("{base_url}/1/messages.json"),
            json!({
                "token": secret, "user": topic.unwrap_or_default(), "title": title, "message": text,
                "priority": pushover_priority(&m.severity), "url": m.link, "url_title": "Open in FinStats",
            }),
        ),
        // A push with somewhere to go is a link; one without is a note, rather than a link to nowhere.
        Channel::Pushbullet => (
            format!("{base_url}/v2/pushes"),
            json!({
                "type": if m.link.is_some() { "link" } else { "note" },
                "title": title, "body": text, "url": m.link,
            }),
        ),
        // Both of these take their title in the body rather than in a header: a title is a film title,
        // and a film title is not always ASCII.
        Channel::Ntfy => (
            base_url.to_string(),
            json!({
                "topic": topic.unwrap_or_default(), "title": title, "message": text,
                "priority": priority(&m.severity), "tags": tags(&m.severity), "click": m.link,
            }),
        ),
        Channel::Gotify => (
            format!("{base_url}/message"),
            json!({
                "title": title, "message": text, "priority": priority(&m.severity),
                "extras": { "client::display": { "contentType": "text/plain" },
                            "client::notification": m.link.as_ref().map(|l| json!({ "click": { "url": l } })) },
            }),
        ),
    })
}

/// Send one message to one destination. The address never appears in what comes back from here.
pub async fn send(app: &App, t: &Target, m: &Message) -> Result<(), SendError> {
    // A personal destination may only ever point at a public host, and a name that answered publicly
    // yesterday can be pointed inwards today, so this is checked again on every send, not only on save.
    let checked = match t.owner_id {
        Some(_) => Some(crate::notify::public_addresses(t.url()).await.map_err(|e| SendError::of(format!("{e}")))?),
        None => None,
    };
    let Some((url, body)) = payload(t.channel, t.url(), t.topic.as_deref(), t.secret(), m) else {
        // Email: a different transport, and the only one that is not a POST of JSON.
        return crate::mail::deliver(
            crate::mail::Account {
                url: t.url(),
                from: t.option("from"),
                to: t.topic.as_deref().unwrap_or_default(),
                username: t.option("username"),
                password: t.secret(),
                lax: t.accept_invalid_certs,
                connect_to: checked.as_ref().and_then(|a| a.first().copied()),
            },
            m,
        )
        .await;
    };
    let client = if checked.is_some() { app.services_http.public(t.accept_invalid_certs) } else { app.services_http.client(t.accept_invalid_certs) };
    let mut req = client.post(&url).json(&body);
    // Where each service wants its token. Telegram's is the path and Pushover's is in the body, both
    // built above; these are the ones that go in a header.
    match t.channel {
        Channel::Gotify => req = req.header("X-Gotify-Key", t.secret()),
        Channel::Pushbullet => req = req.header("Access-Token", t.secret()),
        Channel::Webhook | Channel::Ntfy if !t.secret().is_empty() => req = req.header("Authorization", format!("Bearer {}", t.secret())),
        _ => {}
    }
    let resp = req.send().await.map_err(|e| SendError::of(explain(e, t)))?;
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let retry_after = resp.headers().get(RETRY_AFTER).and_then(|v| v.to_str().ok()).and_then(|v| v.trim().parse::<i64>().ok());
    Err(SendError { message: refuse(status, t), retry_after })
}

/// Why a request never got an answer, in words, and without the URL reqwest would print: that URL is the
/// credential. Errors from here are shown in the UI and written to the log.
fn explain(e: reqwest::Error, target: &Target) -> String {
    let key = if e.is_timeout() {
        "timeout"
    } else if e.is_connect() {
        "connect"
    } else if e.is_decode() {
        "decode"
    } else {
        "unreachable"
    };
    t(&format!("server.channels.{key}"), &[("channel", target.channel.title().into()), ("host", crate::outbound::host_of(target.url()).into())])
}

/// An answer that is not the one asked for: the status and what it usually means, never the body.
fn refuse(status: StatusCode, target: &Target) -> String {
    let key = if status.is_redirection() {
        "redirect"
    } else {
        match status.as_u16() {
            // Telegram and Pushover both answer 400 for a chat or a user key they do not know, and the one
            // the owner just typed is far more likely to be wrong than the token they pasted.
            400 => match target.channel {
                Channel::Telegram => "telegram_chat",
                Channel::Pushover => "pushover_key",
                _ => "not_understood",
            },
            401 | 403 => match target.channel {
                Channel::Discord => "discord_refused",
                Channel::Telegram => "telegram_refused",
                Channel::Pushbullet => "pushbullet_refused",
                _ => "token_refused",
            },
            404 => match target.channel {
                Channel::Slack => "slack_gone",
                _ => "not_found",
            },
            413 => "too_large",
            429 => "rate_limited",
            _ => "answered",
        }
    };
    let host = crate::outbound::host_of(target.url());
    t(&format!("server.channels.{key}"), &[("channel", target.channel.title().into()), ("host", host.into()), ("status", status.to_string().into())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notify::INFO;

    fn msg() -> Message {
        Message {
            kind: crate::notify::Kind::Travel,
            at: 1000,
            user: Some("alice".into()),
            title: "Impossible travel for alice".into(),
            body: "Oslo and London, 40 minutes apart".into(),
            link: Some("https://finstats.example/security".into()),
            severity: ALERT.into(),
            fields: vec![("Person".into(), "alice".into()), ("Places".into(), "Oslo → London".into())],
        }
    }

    /// The payload of a channel that posts JSON, which is every one of them but email.
    fn post(channel: Channel, base: &str, topic: Option<&str>, m: &Message) -> (String, Value) {
        payload(channel, base, topic, "s3cret-token", m).expect("an HTTP channel has a payload")
    }

    /// Every refusal is said in the words that fit the channel, and never with the body the service answered.
    #[test]
    fn a_refusal_says_what_it_usually_means_for_that_channel() {
        let t = |channel: Channel, url: &str| crate::notify::test_target(channel, url);
        let said = |status: u16, channel: Channel, url: &str| refuse(StatusCode::from_u16(status).unwrap(), &t(channel, url));
        assert!(said(307, Channel::Webhook, "https://hooks.example/x").contains("follows none"), "a redirect is never followed, and says so");
        assert!(said(400, Channel::Telegram, "https://api.telegram.org").contains("does not know that chat"));
        assert!(said(400, Channel::Pushover, "https://api.pushover.net").contains("user key or the application token"));
        assert!(said(400, Channel::Webhook, "https://hooks.example/x").ends_with("did not understand the message"));
        assert!(said(401, Channel::Discord, "https://discord.com/api/webhooks/1/t").contains("Has it been deleted?"));
        assert!(said(403, Channel::Telegram, "https://api.telegram.org").ends_with("refused the bot token"));
        assert!(said(401, Channel::Pushbullet, "https://api.pushbullet.com").ends_with("refused the access token"));
        assert!(said(401, Channel::Gotify, "https://gotify.example").ends_with("refused the token"));
        assert!(said(404, Channel::Slack, "https://hooks.slack.com/services/x").contains("no longer knows that webhook"));
        let missing = said(404, Channel::Ntfy, "https://ntfy.example/topic");
        assert!(missing.starts_with("ntfy.example answered 404") && missing.contains("topic and base path"), "{missing}");
        assert!(said(413, Channel::Webhook, "https://hooks.example/x").ends_with("found the message too large"));
        assert!(said(429, Channel::Webhook, "https://hooks.example/x").ends_with("is rate-limiting FinStats"));
        assert!(said(500, Channel::Webhook, "https://hooks.example/x").ends_with("answered 500 Internal Server Error"));
    }

    #[test]
    fn a_severity_is_carried_in_each_service_s_own_scale() {
        assert_eq!([priority(ALERT), priority(WARN), priority(INFO)], [5, 4, 3]);
        assert_eq!([tags(ALERT), tags(WARN), tags(INFO)], [vec!["rotating_light"], vec!["warning"], vec!["information_source"]]);
        // Pushover's 2 needs acknowledging and is never sent; its quietest everyday level is -1.
        assert_eq!([pushover_priority(ALERT), pushover_priority(WARN), pushover_priority(INFO)], [1, 0, -1]);
    }

    #[test]
    fn every_channel_is_posted_the_way_it_expects() {
        let m = msg();
        let (url, body) = post(Channel::Webhook, "https://example.com/hook", None, &m);
        assert_eq!(url, "https://example.com/hook");
        assert_eq!(body["event"], "travel");
        assert_eq!(body["fields"][1]["label"], "Places");

        let (url, body) = post(Channel::Discord, "https://discord.com/api/webhooks/1/tok", None, &m);
        assert_eq!(url, "https://discord.com/api/webhooks/1/tok", "Discord is posted to exactly the webhook URL");
        assert_eq!(body["embeds"][0]["title"], "Impossible travel for alice");
        assert_eq!(body["embeds"][0]["color"], colour(ALERT));

        let (url, body) = post(Channel::Slack, "https://hooks.slack.com/services/T0/B0/tok", None, &m);
        assert_eq!(url, "https://hooks.slack.com/services/T0/B0/tok");
        assert_eq!(body["text"], "Impossible travel for alice", "what a Slack notification shows before the card");
        assert_eq!(body["attachments"][0]["color"], "#e5484d");
        assert_eq!(body["attachments"][0]["title_link"], "https://finstats.example/security");
        assert_eq!(body["attachments"][0]["fields"][0]["title"], "Person");

        let (url, body) = post(Channel::Telegram, "https://api.telegram.org", Some("-1001234567890"), &m);
        assert_eq!(url, "https://api.telegram.org/bots3cret-token/sendMessage", "the bot token is the path");
        assert_eq!(body["chat_id"], "-1001234567890");
        assert!(body.get("parse_mode").is_none(), "a film title is not markup, so none of it is parsed as any");
        let text = body["text"].as_str().unwrap();
        assert!(text.starts_with("Impossible travel for alice\n") && text.contains("Person: alice"));

        let (url, body) = post(Channel::Pushover, "https://api.pushover.net", Some("uQiRzpo4DXghDmr9QzzfQu27cmVRsG"), &m);
        assert_eq!(url, "https://api.pushover.net/1/messages.json");
        assert_eq!(body["token"], "s3cret-token", "the application token travels in the body");
        assert_eq!(body["user"], "uQiRzpo4DXghDmr9QzzfQu27cmVRsG");
        assert_eq!(body["priority"], 1, "an alert is the loud one, never Pushover's emergency");
        assert_eq!(body["url"], "https://finstats.example/security");

        let (url, body) = post(Channel::Pushbullet, "https://api.pushbullet.com", None, &m);
        assert_eq!(url, "https://api.pushbullet.com/v2/pushes");
        assert_eq!(body["type"], "link", "with a link it is a link, and a note without one");
        assert_eq!(body["url"], "https://finstats.example/security");
        assert_eq!(body["title"], "Impossible travel for alice");

        let (url, body) = post(Channel::Ntfy, "https://ntfy.sh", Some("finstats-abc"), &m);
        assert_eq!(url, "https://ntfy.sh", "the topic travels in the body, not in the path");
        assert_eq!(body["topic"], "finstats-abc");
        assert_eq!(body["priority"], 5);
        assert!(body["message"].as_str().unwrap().contains("Person: alice"), "a channel without fields still says everything");

        let (url, body) = post(Channel::Gotify, "http://nas:8070", None, &m);
        assert_eq!(url, "http://nas:8070/message");
        assert_eq!(body["extras"]["client::notification"]["click"]["url"], "https://finstats.example/security");

        assert!(payload(Channel::Email, "smtps://smtp.example.com", Some("me@example.com"), "", &m).is_none(), "email is not an HTTP request at all");
    }

    #[test]
    fn a_push_without_a_link_is_a_note_rather_than_a_link_to_nowhere() {
        let m = Message { link: None, ..msg() };
        let (_, body) = post(Channel::Pushbullet, "https://api.pushbullet.com", None, &m);
        assert_eq!(body["type"], "note");
        assert!(body["url"].is_null());
    }

    #[test]
    fn every_channel_says_what_it_asks_for_and_nothing_more() {
        for c in Channel::ALL {
            assert_eq!(Channel::from_key(c.key()), Some(c), "{}: a key round-trips", c.key());
            assert!(!c.label().is_empty() && !c.what().is_empty(), "{}", c.key());
            // A channel whose address is its service's own asks nobody to type one, and the other way round.
            assert_eq!(c.fixed_url().is_some(), c.example().is_empty(), "{}", c.key());
            assert!(c.secret_required() <= c.secret_label().is_some(), "{}: a token that is required is asked for", c.key());
        }
        // Telegram carries the bot token in the path and says which chat in the body: both are asked for.
        assert_eq!(Channel::Telegram.fixed_url(), Some("https://api.telegram.org"));
        assert_eq!(Channel::Telegram.topic_label(), Some("Chat ID"));
        assert!(Channel::Telegram.secret_required());
        assert_eq!(Channel::Slack.topic_label(), None, "a Slack webhook already says which channel it is for");
        assert_eq!(Channel::Pushover.topic_label(), Some("User or group key"));
        assert_eq!(Channel::Email.extras().iter().map(|e| e.key).collect::<Vec<_>>(), ["from", "username"]);
        assert_eq!(
            Channel::ALL.into_iter().filter(|c| !c.extras().is_empty()).collect::<Vec<_>>(),
            vec![Channel::Email],
            "only email needs more than an address, a token and one field beside them"
        );
    }

    #[test]
    fn a_long_title_is_cut_rather_than_refused() {
        let long = "a".repeat(400);
        let m = Message { kind: crate::notify::Kind::NewItems, at: 1, user: None, title: long, body: "b".repeat(9_000), link: None, severity: INFO.into(), fields: vec![] };
        let (_, body) = post(Channel::Discord, "https://discord.com/api/webhooks/1/t", None, &m);
        assert!(body["embeds"][0]["title"].as_str().unwrap().chars().count() <= TITLE_MAX);
        assert!(body["embeds"][0]["description"].as_str().unwrap().chars().count() <= BODY_MAX);
    }
}

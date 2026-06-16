#![cfg(any(
    feature = "rustls-ring",
    feature = "rustls-aws",
    feature = "native-tls"
))]
//! End-to-end Gmail REST API test.
//!
//! Requires an OAuth2 access token with read/write/send scopes:
//!
//! ```sh
//! GMAIL_ACCESS_TOKEN="<token>" \
//! cargo test --test gmail -- --include-ignored
//! ```
//!
//! `GMAIL_USER_ID` is optional and defaults to `me`.

use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use io_gmail::{client::GmailClientStd, messages::GmailMessageFormat, send::GMAIL_API_BASE};
use pimalaya_stream::tls::Tls;
use secrecy::SecretString;
use url::Url;

#[test]
#[ignore = "requires GMAIL_ACCESS_TOKEN env var and --include-ignored"]
fn gmail() {
    env_logger::try_init().ok();

    let token = env::var("GMAIL_ACCESS_TOKEN").expect("GMAIL_ACCESS_TOKEN not set");
    let user_id = env::var("GMAIL_USER_ID").unwrap_or_else(|_| "me".to_owned());
    let http_auth = SecretString::from(format!("Bearer {token}"));

    let url = Url::parse(GMAIL_API_BASE).expect("parse Gmail API base URL");
    let tls = Tls::default();

    let mut client =
        GmailClientStd::connect(&url, &tls, http_auth.clone(), user_id.clone()).expect("connect");

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let label_name = format!("io-gmail-test-{ts}");
    let label_name_renamed = format!("{label_name}-renamed");

    // ── PROFILE GET ──────────────────────────────────────────────────────────

    let profile = client.profile_get().expect("profile get").response;
    let email = profile.email_address.clone();
    assert!(!email.is_empty(), "profile should expose an email address");

    // ── LABELS LIST (baseline) ───────────────────────────────────────────────

    let labels = client.labels_list().expect("labels list").response;
    assert!(
        labels.labels.iter().any(|label| label.id == "INBOX"),
        "labels list should contain the INBOX system label"
    );

    // ── LABEL CREATE ─────────────────────────────────────────────────────────

    let label = client
        .label_create(&label_name)
        .expect("label create")
        .response;
    let label_id = label.id.clone();
    assert_eq!(label.name, label_name, "created label name mismatch");

    // ── LABEL GET (verify creation) ──────────────────────────────────────────

    let fetched = client.label_get(&label_id).expect("label get").response;
    assert_eq!(fetched.id, label_id, "label get id mismatch");

    // ── LABEL UPDATE (rename) ────────────────────────────────────────────────

    let renamed = client
        .label_update(&label_id, &label_name_renamed)
        .expect("label update")
        .response;
    assert_eq!(
        renamed.name, label_name_renamed,
        "label rename not reflected"
    );

    // ── MESSAGE SEND ─────────────────────────────────────────────────────────

    let eml = build_eml(&email).into_bytes();
    let sent = client.message_send(&eml).expect("message send").response;
    let message_id = sent.id.clone();

    // ── MESSAGE GET (verify send) ────────────────────────────────────────────

    let message = client
        .message_get(&message_id, GmailMessageFormat::Full, &[])
        .expect("message get")
        .response;
    assert_eq!(message.id, message_id, "message get id mismatch");

    // ── MESSAGE MODIFY (add then remove the test label) ──────────────────────

    let labelled = client
        .message_modify(&message_id, std::slice::from_ref(&label_id), &[])
        .expect("message modify add")
        .response;
    assert!(
        labelled.label_ids.contains(&label_id),
        "message should carry the test label after modify"
    );

    let unlabelled = client
        .message_modify(&message_id, &[], std::slice::from_ref(&label_id))
        .expect("message modify remove")
        .response;
    assert!(
        !unlabelled.label_ids.contains(&label_id),
        "message should not carry the test label after removal"
    );

    // ── MESSAGES LIST (find the sent message) ────────────────────────────────

    let listed = client
        .messages_list(Some("subject:io-gmail"), &[], Some(10), None, true)
        .expect("messages list")
        .response;
    assert!(
        listed.messages.iter().any(|m| m.id == message_id),
        "messages list should surface the sent message"
    );

    // ── MESSAGE TRASH then UNTRASH ───────────────────────────────────────────

    let trashed = client
        .message_trash(&message_id)
        .expect("message trash")
        .response;
    assert!(
        trashed.label_ids.iter().any(|id| id == "TRASH"),
        "trashed message should carry the TRASH label"
    );

    let untrashed = client
        .message_untrash(&message_id)
        .expect("message untrash")
        .response;
    assert!(
        !untrashed.label_ids.iter().any(|id| id == "TRASH"),
        "untrashed message should no longer carry the TRASH label"
    );

    // ── CLEANUP: delete the message then the label ───────────────────────────

    client.message_delete(&message_id).expect("message delete");
    client.label_delete(&label_id).expect("label delete");
}

fn build_eml(email: &str) -> String {
    [
        &format!("From: io-gmail test <{email}>"),
        &format!("To: io-gmail test <{email}>"),
        "Subject: io-gmail integration test",
        "Date: Thu, 01 Jan 2026 00:00:00 +0000",
        "MIME-Version: 1.0",
        "Content-Type: text/plain; charset=utf-8",
        "",
        "This is an automated test email from io-gmail integration tests.",
    ]
    .join("\r\n")
}

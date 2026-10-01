//! End-to-end Gmail REST API test.
//!
//! Google only accepts an OAuth2 Bearer token, and there are two ways
//! to hand one to this test.
//!
//! A token minted by hand, with read/write/send scopes, acts as you and
//! dies within the hour:
//!
//! ```sh
//! GMAIL_ACCESS_TOKEN="<token>" \
//! cargo test --test gmail -- --include-ignored
//! ```
//!
//! A Workspace service account with domain-wide delegation instead
//! signs its own assertion on behalf of a user of the domain, so the
//! run needs no human. The account owns no mailbox of its own, hence
//! the subject naming the user whose mailbox the test borrows:
//!
//! ```sh
//! GMAIL_SERVICE_ACCOUNT_KEY_FILE=key.json \
//! GMAIL_SERVICE_ACCOUNT_SUBJECT=google@pimalaya.org \
//! cargo test --test gmail -- --include-ignored
//! ```
//!
//! CI passes the key itself rather than a path, as
//! `GMAIL_SERVICE_ACCOUNT_KEY`, since it comes straight out of a secret.
//! The subject defaults to `google@pimalaya.org`, the Pimalaya test user.
//! The delegation must grant the `https://mail.google.com/` scope.
//!
//! `GMAIL_USER_ID` is optional and defaults to `me`.
//!
//! Everything the run creates is named after one `io-gmail-test-<millis>`
//! tag, and [`with_cleanup`] deletes it however the run ends. Messages
//! the run sends land in the borrowed mailbox, since they are sent to
//! the mailbox itself, and are deleted with the rest.
//!
//! The settings and the Pub/Sub `watch` are deliberately left out: the
//! former change the whole mailbox, the latter needs a Pub/Sub topic.

#![cfg(any(
    feature = "rustls-ring",
    feature = "rustls-aws",
    feature = "native-tls"
))]

use core::fmt::Debug;

use std::{
    borrow::Cow,
    env, fs,
    io::{Read, Write},
    panic::{self, AssertUnwindSafe},
    slice, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use io_gmail::{
    coroutine::*,
    v1::{
        client::{GmailClientStd, GmailClientStdConnectOptions},
        history_poll::{GmailHistoryPoll, GmailHistoryPollYield},
        rest::{
            drafts::{
                GmailDraft,
                create::GmailDraftCreate,
                delete::GmailDraftDelete,
                get::GmailDraftGet,
                list::{GmailDraftsList, GmailDraftsListParams},
                send::GmailDraftSend,
                update::GmailDraftUpdate,
            },
            history::{
                GmailHistoryType,
                list::{GmailHistoryList, GmailHistoryListParams},
            },
            labels::{GmailLabel, GmailLabelListVisibility},
            messages::{
                GmailInternalDateSource, GmailMessage, GmailMessageFormat, GmailMessageId,
                GmailMessageListVisibility, GmailMessagePayload,
                attachments::get::GmailAttachmentGet, batch_delete::GmailMessagesBatchDelete,
                batch_modify::GmailMessagesBatchModify, decode_raw, encode_raw,
                import::GmailMessageImport, insert::GmailMessageInsert,
                list::GmailMessagesListParams,
            },
            threads::{
                delete::GmailThreadDelete,
                get::GmailThreadGet,
                list::{GmailThreadsList, GmailThreadsListParams},
                modify::GmailThreadModify,
                trash::GmailThreadTrash,
                untrash::GmailThreadUntrash,
            },
        },
    },
};
use io_oauth::{
    client::Oauth20ClientStd,
    rfc7523::{
        assertion::{Oauth20JwtBearerClaims, Oauth20JwtBearerKey},
        auth_grant::Oauth20JwtBearerGrantRequestParams,
    },
};
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;
use serde::Deserialize;
use url::Url;

/// The scope the test needs: full access to the mailbox, the one Gmail
/// scope that covers labels, sending and permanent deletion.
const GMAIL_SCOPE: &str = "https://mail.google.com/";

/// The Pimalaya Workspace user the service account acts as when
/// `GMAIL_SERVICE_ACCOUNT_SUBJECT` names none.
const DEFAULT_SUBJECT: &str = "google@pimalaya.org";

/// The `Date` header of every message the run builds.
const DATE: &str = "Thu, 01 Jan 2026 00:00:00 +0000";

/// [`DATE`] in epoch milliseconds, as Gmail reports an internal date.
const DATE_MILLIS: &str = "1767225600000";

/// The attachment the sent message carries, and its base64 encoding.
const ATTACHMENT: &[u8] = b"io-gmail attachment\n";
const ATTACHMENT_BASE64: &str = "aW8tZ21haWwgYXR0YWNobWVudAo=";

#[test]
#[ignore = "requires GMAIL_ACCESS_TOKEN or a service account key, and --include-ignored"]
fn gmail() {
    let mut client = connect();
    let tag = format!("io-gmail-test-{}", unix_millis());
    let mut leftovers = Leftovers::default();

    let start_history_id = client
        .profile_get()
        .expect("profile get")
        .response
        .history_id
        .expect("profile should expose a history id");

    with_cleanup(
        &mut client,
        &mut leftovers,
        |client, leftovers| {
            let email = profile(client);
            let label_id = labels(client, &tag, leftovers);
            let sent = messages(client, &email, &tag, &label_id, leftovers);
            threads(client, &sent, &label_id, leftovers);
            insert_and_import(client, &email, &tag, &label_id, leftovers);
            history_poll(client, &email, &tag, &label_id, leftovers);
            drafts(client, &email, &tag, leftovers);
        },
        |client, leftovers| {
            for id in &leftovers.drafts {
                let coroutine = GmailDraftDelete::new(&client.auth, &client.user_id, id)
                    .expect("draft delete coroutine");
                if let Err(err) = client.run(coroutine) {
                    report_leftover("draft", id, &err);
                }
            }

            for id in &leftovers.messages {
                if let Err(err) = client.message_delete(id) {
                    report_leftover("message", id, &err);
                }
            }

            // NOTE: a message sent to the mailbox itself is delivered
            // back a moment later, and lands as a new message when the
            // run deleted the sent one by then. Nothing signals that
            // delivery, so the history since the run started is read a
            // few times, and whatever it added under the run's tag is
            // deleted.
            for _ in 0..3 {
                thread::sleep(Duration::from_secs(3));

                let coroutine = GmailHistoryList::new(
                    &client.auth,
                    &client.user_id,
                    &GmailHistoryListParams {
                        start_history_id: &start_history_id,
                        history_types: &[GmailHistoryType::MessageAdded],
                        max_results: Some(500),
                        ..Default::default()
                    },
                )
                .expect("history list coroutine");
                let history = client.run(coroutine).expect("history list").response;

                for added in history.history.iter().flat_map(|r| &r.messages_added) {
                    let id = &added.message.id;

                    if subject(client, id).is_some_and(|s| s.starts_with(&tag))
                        && let Err(err) = client.message_delete(id)
                    {
                        report_leftover("message", id, &err);
                    }
                }
            }

            if let Some(id) = &leftovers.label
                && let Err(err) = client.label_delete(id)
            {
                report_leftover("label", id, &err);
            }
        },
    );
}

/// Reads the profile of the borrowed mailbox and returns its address.
fn profile(client: &mut GmailClientStd) -> String {
    let profile = client.profile_get().expect("profile get").response;
    assert!(
        !profile.email_address.is_empty(),
        "profile should expose an email address"
    );
    assert!(
        profile.history_id.is_some(),
        "profile should expose a history id"
    );

    profile.email_address
}

/// Creates, reads, renames and patches the test label, and returns its
/// id.
fn labels(client: &mut GmailClientStd, tag: &str, leftovers: &mut Leftovers) -> String {
    let labels = client.labels_list().expect("labels list").response;
    assert!(
        labels.labels.iter().any(|label| label.id == "INBOX"),
        "labels list should contain the INBOX system label"
    );

    let label = client
        .label_create(&GmailLabel {
            name: tag.to_owned(),
            ..Default::default()
        })
        .expect("label create")
        .response;
    let id = label.id.clone();
    leftovers.label = Some(id.clone());
    assert_eq!(label.name, tag, "created label name mismatch");

    let fetched = client.label_get(&id).expect("label get").response;
    assert_eq!(fetched.id, id, "label get id mismatch");

    let renamed_name = format!("{tag}-renamed");
    let renamed = client
        .label_update(&GmailLabel {
            id: id.clone(),
            name: renamed_name.clone(),
            ..Default::default()
        })
        .expect("label update")
        .response;
    assert_eq!(renamed.name, renamed_name, "label rename not reflected");

    let patched = client
        .label_patch(&GmailLabel {
            id: id.clone(),
            label_list_visibility: Some(GmailLabelListVisibility::LabelHide),
            message_list_visibility: Some(GmailMessageListVisibility::Hide),
            ..Default::default()
        })
        .expect("label patch")
        .response;
    assert_eq!(
        patched.label_list_visibility,
        Some(GmailLabelListVisibility::LabelHide)
    );
    assert_eq!(
        patched.name, renamed_name,
        "a patch leaves the fields it does not carry alone"
    );

    id
}

/// Sends a message with an attachment to the mailbox itself, reads it
/// and its attachment back, labels, lists, trashes and untrashes it.
fn messages(
    client: &mut GmailClientStd,
    email: &str,
    tag: &str,
    label_id: &str,
    leftovers: &mut Leftovers,
) -> GmailMessageId {
    let eml = eml_with_attachment(email, &format!("{tag} sent"));
    let sent = client
        .message_send(&GmailMessage {
            raw: Some(encode_raw(eml.as_bytes())),
            ..Default::default()
        })
        .expect("message send")
        .response;
    let id = sent.id.clone();
    leftovers.messages.push(id.clone());

    let message = client
        .message_get(&id, GmailMessageFormat::Full, &[])
        .expect("message get")
        .response;
    assert_eq!(message.id, id, "message get id mismatch");

    let attachment_id = message
        .payload
        .as_ref()
        .and_then(|payload| find_part(payload, "io-gmail.txt"))
        .and_then(|part| part.body.as_ref())
        .and_then(|body| body.attachment_id.clone())
        .expect("the attachment part carries an attachment id");

    let coroutine = GmailAttachmentGet::new(&client.auth, &client.user_id, &id, &attachment_id)
        .expect("attachment get coroutine");
    let attachment = client.run(coroutine).expect("attachment get").response;
    let data = attachment.data.expect("the attachment carries its data");
    assert_eq!(
        decode_raw(&data).expect("the attachment data is base64url"),
        ATTACHMENT
    );

    let labelled = client
        .message_modify(&id, &[label_id.to_owned()], &[])
        .expect("message modify add")
        .response;
    assert!(
        labelled.label_ids.iter().any(|l| l == label_id),
        "message should carry the test label after modify"
    );

    let unlabelled = client
        .message_modify(&id, &[], &[label_id.to_owned()])
        .expect("message modify remove")
        .response;
    assert!(
        !unlabelled.label_ids.iter().any(|l| l == label_id),
        "message should not carry the test label after removal"
    );

    let listed = client
        .messages_list(&GmailMessagesListParams {
            q: Some("subject:io-gmail"),
            max_results: Some(10),
            include_spam_trash: true,
            ..Default::default()
        })
        .expect("messages list")
        .response;
    assert!(
        listed.messages.iter().any(|m| m.id == id),
        "messages list should surface the sent message"
    );

    let trashed = client.message_trash(&id).expect("message trash").response;
    assert!(
        trashed.label_ids.iter().any(|l| l == "TRASH"),
        "trashed message should carry the TRASH label"
    );

    let untrashed = client
        .message_untrash(&id)
        .expect("message untrash")
        .response;
    assert!(
        !untrashed.label_ids.iter().any(|l| l == "TRASH"),
        "untrashed message should no longer carry the TRASH label"
    );

    sent
}

/// Walks the thread of the sent message: labels, finds, reads,
/// unlabels, trashes, untrashes, then deletes it with its message.
fn threads(
    client: &mut GmailClientStd,
    sent: &GmailMessageId,
    label_id: &str,
    leftovers: &mut Leftovers,
) {
    let thread_id = sent
        .thread_id
        .as_deref()
        .expect("the sent message carries a thread id");
    let label_ids = [label_id.to_owned()];

    let coroutine =
        GmailThreadModify::new(&client.auth, &client.user_id, thread_id, &label_ids, &[])
            .expect("thread modify coroutine");
    client.run(coroutine).expect("thread modify add");

    // NOTE: filtered by label rather than by query, since a label
    // applies at once while the search index lags behind.
    let coroutine = GmailThreadsList::new(
        &client.auth,
        &client.user_id,
        &GmailThreadsListParams {
            label_ids: &label_ids,
            max_results: Some(10),
            ..Default::default()
        },
    )
    .expect("threads list coroutine");
    let listed = client.run(coroutine).expect("threads list").response;
    assert!(
        listed.threads.iter().any(|t| t.id == thread_id),
        "threads list should surface the labelled thread"
    );

    let coroutine = GmailThreadGet::new(
        &client.auth,
        &client.user_id,
        thread_id,
        GmailMessageFormat::Minimal,
        &[],
    )
    .expect("thread get coroutine");
    let thread = client.run(coroutine).expect("thread get").response;
    let message = thread
        .messages
        .iter()
        .find(|m| m.id == sent.id)
        .expect("the thread holds the sent message");
    assert!(
        message.label_ids.iter().any(|l| l == label_id),
        "the thread label applies to its messages"
    );

    let coroutine =
        GmailThreadModify::new(&client.auth, &client.user_id, thread_id, &[], &label_ids)
            .expect("thread modify coroutine");
    client.run(coroutine).expect("thread modify remove");

    let coroutine = GmailThreadTrash::new(&client.auth, &client.user_id, thread_id)
        .expect("thread trash coroutine");
    let trashed = client.run(coroutine).expect("thread trash").response;
    assert!(
        trashed
            .messages
            .iter()
            .all(|m| m.label_ids.iter().any(|l| l == "TRASH")),
        "every message of a trashed thread carries the TRASH label"
    );

    let coroutine = GmailThreadUntrash::new(&client.auth, &client.user_id, thread_id)
        .expect("thread untrash coroutine");
    client.run(coroutine).expect("thread untrash");

    let coroutine = GmailThreadDelete::new(&client.auth, &client.user_id, thread_id)
        .expect("thread delete coroutine");
    client.run(coroutine).expect("thread delete");
    leftovers.forget_message(&sent.id);
}

/// Inserts then imports a message, sees both in the history since the
/// profile's history id, labels both at once, then deletes them one
/// by one and in batch.
fn insert_and_import(
    client: &mut GmailClientStd,
    email: &str,
    tag: &str,
    label_id: &str,
    leftovers: &mut Leftovers,
) {
    let start_history_id = client
        .profile_get()
        .expect("profile get")
        .response
        .history_id
        .expect("profile should expose a history id");

    let message = GmailMessage {
        raw: Some(encode_raw(
            eml(email, &format!("{tag} inserted")).as_bytes(),
        )),
        label_ids: vec![String::from("INBOX")],
        ..Default::default()
    };
    let coroutine = GmailMessageInsert::new(
        &client.auth,
        &client.user_id,
        &message,
        Some(GmailInternalDateSource::DateHeader),
        false,
    )
    .expect("message insert coroutine");
    client.run(coroutine).expect("message insert");

    let message = GmailMessage {
        raw: Some(encode_raw(
            eml(email, &format!("{tag} imported")).as_bytes(),
        )),
        label_ids: vec![String::from("INBOX")],
        ..Default::default()
    };
    let coroutine = GmailMessageImport::new(
        &client.auth,
        &client.user_id,
        &message,
        Some(GmailInternalDateSource::DateHeader),
        false,
        true,
        false,
    )
    .expect("message import coroutine");
    client.run(coroutine).expect("message import");

    // NOTE: the id an import answers is not always the id the message
    // ends up with (the answered one may 404 right after), so both
    // messages are found back by subject among the ones the history
    // reports added.
    let coroutine = GmailHistoryList::new(
        &client.auth,
        &client.user_id,
        &GmailHistoryListParams {
            start_history_id: &start_history_id,
            history_types: &[GmailHistoryType::MessageAdded],
            ..Default::default()
        },
    )
    .expect("history list coroutine");
    let history = client.run(coroutine).expect("history list").response;
    let added: Vec<String> = history
        .history
        .iter()
        .flat_map(|record| &record.messages_added)
        .map(|added| added.message.id.clone())
        .collect();

    let mut inserted = None;
    let mut imported = None;

    for id in added {
        let subject = subject(client, &id).unwrap_or_default();

        if subject == format!("{tag} inserted") {
            inserted = Some(id);
        } else if subject == format!("{tag} imported") {
            imported = Some(id);
        }
    }

    let inserted = inserted.expect("the history reports the inserted message");
    let imported = imported.expect("the history reports the imported message");
    leftovers.messages.push(inserted.clone());
    leftovers.messages.push(imported.clone());

    let message = client
        .message_get(&inserted, GmailMessageFormat::Minimal, &[])
        .expect("message get")
        .response;
    assert_eq!(
        message.internal_date.as_deref(),
        Some(DATE_MILLIS),
        "the internal date comes from the Date header"
    );

    let ids = [inserted.clone(), imported.clone()];
    let coroutine = GmailMessagesBatchModify::new(
        &client.auth,
        &client.user_id,
        &ids,
        &[label_id.to_owned()],
        &[],
    )
    .expect("messages batch modify coroutine");
    client.run(coroutine).expect("messages batch modify");

    for id in &ids {
        let message = client
            .message_get(id, GmailMessageFormat::Minimal, &[])
            .expect("message get")
            .response;
        assert!(
            message.label_ids.iter().any(|l| l == label_id),
            "batch modify labels every message"
        );
    }

    client.message_delete(&inserted).expect("message delete");
    leftovers.forget_message(&inserted);

    let coroutine =
        GmailMessagesBatchDelete::new(&client.auth, &client.user_id, slice::from_ref(&imported))
            .expect("messages batch delete coroutine");
    client.run(coroutine).expect("messages batch delete");
    leftovers.forget_message(&imported);
}

/// Watches the test label through the history poll: once the poll has
/// baselined, inserts a labelled message and waits for the diff adding
/// it, then deletes it and waits for the diff removing it.
///
/// The poll asks for a 30-second sleep between ticks; the test performs
/// its change instead on the first one, and only sleeps briefly on the
/// next ones, while the history catches up.
fn history_poll(
    client: &mut GmailClientStd,
    email: &str,
    tag: &str,
    label_id: &str,
    leftovers: &mut Leftovers,
) {
    let subject = format!("{tag} polled");
    let mut poll = GmailHistoryPoll::new(&client.auth, &client.user_id, label_id)
        .expect("history poll coroutine");

    let mut buf = [0u8; 16 * 1024];
    let mut arg: Option<&[u8]> = None;
    let mut added: Option<String> = None;
    let mut removed = false;
    let mut ticks = 0;

    while !removed {
        match poll.resume(arg.take()) {
            GmailCoroutineState::Complete(Err(err)) => panic!("history poll: {err:?}"),
            GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsRead) => {
                let n = client.stream.read(&mut buf).expect("read");
                arg = Some(&buf[..n]);
            }
            GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsWrite(bytes)) => {
                client.stream.write_all(&bytes).expect("write");
            }
            GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsSleep(_)) if ticks == 0 => {
                ticks += 1;

                let message = GmailMessage {
                    raw: Some(encode_raw(eml(email, &subject).as_bytes())),
                    label_ids: vec![label_id.to_owned()],
                    ..Default::default()
                };
                let coroutine =
                    GmailMessageInsert::new(&client.auth, &client.user_id, &message, None, false)
                        .expect("message insert coroutine");
                client.run(coroutine).expect("message insert");
            }
            GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsSleep(_)) => {
                ticks += 1;
                assert!(ticks <= 10, "the history poll never reported the changes");
                thread::sleep(Duration::from_secs(2));
            }
            GmailCoroutineState::Yielded(GmailHistoryPollYield::Diff(diff)) => {
                assert!(!diff.history_id.is_empty(), "a diff carries its cursor");

                match &added {
                    None => {
                        let message = diff.added.iter().find(|message| {
                            message.payload.iter().any(|payload| {
                                payload
                                    .headers
                                    .iter()
                                    .any(|h| h.name == "Subject" && h.value == subject)
                            })
                        });

                        if let Some(message) = message {
                            let id = message.id.clone();
                            leftovers.messages.push(id.clone());
                            client.message_delete(&id).expect("message delete");
                            leftovers.forget_message(&id);
                            added = Some(id);
                        }
                    }
                    Some(id) => removed = diff.removed.iter().any(|m| &m.id == id),
                }
            }
        }
    }
}

/// Creates, reads, lists, updates and deletes a draft, then sends a
/// second one and deletes the message it became.
fn drafts(client: &mut GmailClientStd, email: &str, tag: &str, leftovers: &mut Leftovers) {
    let subject = format!("{tag} draft");
    let new_draft = draft(email, &subject);
    let coroutine = GmailDraftCreate::new(&client.auth, &client.user_id, &new_draft)
        .expect("draft create coroutine");
    let created = client.run(coroutine).expect("draft create").response;
    let id = created.id.clone();
    leftovers.drafts.push(id.clone());

    let coroutine = GmailDraftGet::new(&client.auth, &client.user_id, &id, GmailMessageFormat::Raw)
        .expect("draft get coroutine");
    let fetched = client.run(coroutine).expect("draft get").response;
    let raw = fetched
        .message
        .and_then(|message| message.raw)
        .expect("a raw draft carries its message");
    let raw = String::from_utf8(decode_raw(&raw).expect("the raw draft is base64url"))
        .expect("the raw draft is UTF-8");
    assert!(raw.contains(&subject), "the draft keeps its subject");

    let coroutine = GmailDraftsList::new(
        &client.auth,
        &client.user_id,
        &GmailDraftsListParams {
            max_results: Some(100),
            ..Default::default()
        },
    )
    .expect("drafts list coroutine");
    let listed = client.run(coroutine).expect("drafts list").response;
    assert!(
        listed.drafts.iter().any(|d| d.id == id),
        "drafts list should surface the created draft"
    );

    let updated_subject = format!("{subject} updated");
    let coroutine = GmailDraftUpdate::new(
        &client.auth,
        &client.user_id,
        &GmailDraft {
            id: id.clone(),
            ..draft(email, &updated_subject)
        },
    )
    .expect("draft update coroutine");
    let updated = client.run(coroutine).expect("draft update").response;
    assert_eq!(updated.id, id, "an update keeps the draft id");

    let coroutine =
        GmailDraftDelete::new(&client.auth, &client.user_id, &id).expect("draft delete coroutine");
    client.run(coroutine).expect("draft delete");
    leftovers.forget_draft(&id);

    let coroutine = GmailDraftCreate::new(
        &client.auth,
        &client.user_id,
        &draft(email, &format!("{tag} draft sent")),
    )
    .expect("draft create coroutine");
    let created = client.run(coroutine).expect("draft create").response;
    leftovers.drafts.push(created.id.clone());

    let coroutine =
        GmailDraftSend::new(&client.auth, &client.user_id, &created).expect("draft send coroutine");
    let sent = client.run(coroutine).expect("draft send").response;
    leftovers.forget_draft(&created.id);
    leftovers.messages.push(sent.id.clone());

    let message = client
        .message_get(&sent.id, GmailMessageFormat::Minimal, &[])
        .expect("message get")
        .response;
    assert!(
        message.label_ids.iter().any(|l| l == "SENT"),
        "a sent draft becomes a SENT message"
    );

    // NOTE: a message a draft send just produced answers a delete with
    // a 500 for a few seconds, which Gmail documents as retryable.
    for attempt in 1.. {
        match client.message_delete(&sent.id) {
            Ok(_) => break,
            Err(_) if attempt < 10 => thread::sleep(Duration::from_secs(2)),
            Err(err) => panic!("message delete: {err:?}"),
        }
    }
    leftovers.forget_message(&sent.id);
}

// --- utils ---------------------------------------------------------------

/// What the run created and has not deleted yet, for [`with_cleanup`]
/// to remove whichever way the run ends.
#[derive(Debug, Default)]
struct Leftovers {
    label: Option<String>,
    messages: Vec<String>,
    drafts: Vec<String>,
}

impl Leftovers {
    fn forget_message(&mut self, id: &str) {
        self.messages.retain(|m| m != id);
    }

    fn forget_draft(&mut self, id: &str) {
        self.drafts.retain(|d| d != id);
    }
}

/// Opens the TCP and TLS connection out of the environment token.
fn connect() -> GmailClientStd {
    env_logger::try_init().ok();

    let options = GmailClientStdConnectOptions {
        user_id: env::var("GMAIL_USER_ID").unwrap_or_else(|_| String::from("me")),
        ..Default::default()
    };

    GmailClientStd::connect(token(), options).expect("connect")
}

/// A plain text message from and to the mailbox itself.
fn eml(email: &str, subject: &str) -> String {
    [
        &format!("From: io-gmail test <{email}>"),
        &format!("To: io-gmail test <{email}>"),
        &format!("Subject: {subject}"),
        &format!("Date: {DATE}"),
        "MIME-Version: 1.0",
        "Content-Type: text/plain; charset=utf-8",
        "",
        "This is an automated test email from io-gmail integration tests.",
    ]
    .join("\r\n")
}

/// A multipart message from and to the mailbox itself, carrying
/// [`ATTACHMENT`] as `io-gmail.txt`.
fn eml_with_attachment(email: &str, subject: &str) -> String {
    [
        &format!("From: io-gmail test <{email}>"),
        &format!("To: io-gmail test <{email}>"),
        &format!("Subject: {subject}"),
        &format!("Date: {DATE}"),
        "MIME-Version: 1.0",
        "Content-Type: multipart/mixed; boundary=io-gmail",
        "",
        "--io-gmail",
        "Content-Type: text/plain; charset=utf-8",
        "",
        "This is an automated test email from io-gmail integration tests.",
        "--io-gmail",
        "Content-Type: text/plain; charset=utf-8; name=io-gmail.txt",
        "Content-Disposition: attachment; filename=io-gmail.txt",
        "Content-Transfer-Encoding: base64",
        "",
        ATTACHMENT_BASE64,
        "--io-gmail--",
    ]
    .join("\r\n")
}

/// A draft holding a plain text message from and to the mailbox itself.
fn draft(email: &str, subject: &str) -> GmailDraft {
    GmailDraft {
        message: Some(GmailMessage {
            raw: Some(encode_raw(eml(email, subject).as_bytes())),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// The `Subject` header of the given message, if it still exists.
fn subject(client: &mut GmailClientStd, id: &str) -> Option<String> {
    let message = client
        .message_get(id, GmailMessageFormat::Metadata, &["Subject"])
        .ok()?
        .response;

    message
        .payload
        .into_iter()
        .flat_map(|payload| payload.headers)
        .find(|header| header.name.eq_ignore_ascii_case("Subject"))
        .map(|header| header.value)
}

/// Finds the part of the given payload tree named `filename`.
fn find_part<'a>(
    payload: &'a GmailMessagePayload,
    filename: &str,
) -> Option<&'a GmailMessagePayload> {
    if payload.filename == filename {
        return Some(payload);
    }

    payload
        .parts
        .iter()
        .find_map(|part| find_part(part, filename))
}

/// Milliseconds since the Unix epoch, to mint a unique tag per run.
fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/// Runs `body`, then `cleanup` whichever way `body` went, and only then
/// re-raises a panic `body` may have raised.
///
/// The flow runs against a real mailbox. Every step panics on failure,
/// so a teardown written as the last statements of the flow is skipped
/// the moment anything goes wrong, and each failed run leaves labels and
/// messages behind for good. `body` records what it creates in `state`,
/// which `cleanup` then removes.
fn with_cleanup<S, B, C>(client: &mut GmailClientStd, state: &mut S, body: B, cleanup: C)
where
    B: FnOnce(&mut GmailClientStd, &mut S),
    C: FnOnce(&mut GmailClientStd, &mut S),
{
    let outcome = panic::catch_unwind(AssertUnwindSafe(|| body(client, state)));

    if panic::catch_unwind(AssertUnwindSafe(|| cleanup(client, state))).is_err() {
        eprintln!("WARNING: cleanup itself failed, the mailbox may hold leftovers");
    }

    if let Err(payload) = outcome {
        panic::resume_unwind(payload);
    }
}

/// Reports a failed teardown without panicking, naming what was left
/// behind so it can be removed by hand.
fn report_leftover(what: &str, id: &str, err: &dyn Debug) {
    eprintln!("WARNING: could not clean up {what} `{id}`, remove it by hand: {err:?}");
}

/// The bearer token the run authenticates with.
///
/// `GMAIL_ACCESS_TOKEN` short-circuits everything when it is set, for
/// the token you minted by hand. Otherwise a service account key, held
/// inline in `GMAIL_SERVICE_ACCOUNT_KEY` or at the path
/// `GMAIL_SERVICE_ACCOUNT_KEY_FILE`, is traded for a fresh token acting
/// as `GMAIL_SERVICE_ACCOUNT_SUBJECT`.
fn token() -> String {
    if let Ok(token) = env::var("GMAIL_ACCESS_TOKEN") {
        return token;
    }

    if let Ok(key) = env::var("GMAIL_SERVICE_ACCOUNT_KEY") {
        return mint_token(&key);
    }

    if let Ok(path) = env::var("GMAIL_SERVICE_ACCOUNT_KEY_FILE") {
        let key = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read the service account key at {path}: {err}"));

        return mint_token(&key);
    }

    panic!(
        "set GMAIL_ACCESS_TOKEN, or GMAIL_SERVICE_ACCOUNT_KEY / \
         GMAIL_SERVICE_ACCOUNT_KEY_FILE to mint one"
    );
}

/// The subset of a service account key file the JWT bearer grant needs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    #[serde(default = "default_token_uri")]
    token_uri: String,
}

fn default_token_uri() -> String {
    String::from("https://oauth2.googleapis.com/token")
}

/// Signs a JWT bearer assertion with the service account key, on behalf
/// of the delegated subject, and trades it for an access token (RFC 7523
/// section 2.1).
///
/// The scopes ride in the claims, which is Google's deviation from the
/// RFC, and io-oauth models it: the token endpoint reads them from there
/// rather than from the request body.
fn mint_token(key: &str) -> String {
    let key: ServiceAccountKey =
        serde_json::from_str(key).expect("the service account key is valid JSON");
    let subject =
        env::var("GMAIL_SERVICE_ACCOUNT_SUBJECT").unwrap_or_else(|_| String::from(DEFAULT_SUBJECT));

    let signer = Oauth20JwtBearerKey::from_pkcs8_pem(&key.private_key)
        .expect("the service account key holds a PKCS#8 private key");

    let token_uri: Url = key.token_uri.parse().expect("the token URI is a valid URL");

    let mut client =
        Oauth20ClientStd::connect(token_uri, &Tls::default(), key.client_email.as_str())
            .expect("connect to the token endpoint");

    let claims = Oauth20JwtBearerClaims {
        iss: key.client_email.as_str().into(),
        sub: Some(subject.into()),
        scope: [Cow::from(GMAIL_SCOPE)].into_iter().collect(),
        ..Default::default()
    };

    // NOTE: iat and exp come from the clock here, in the std client;
    // the coroutine layer underneath stays clock-free.
    let assertion = client
        .sign_jwt_bearer_assertion(&signer, claims, None, Duration::from_secs(600))
        .expect("sign the assertion");

    let params = Oauth20JwtBearerGrantRequestParams {
        assertion,
        scope: Default::default(),
    };

    let response = client
        .request_jwt_bearer_grant(params)
        .expect("trade the assertion for an access token");

    match response {
        Ok(granted) => granted.access_token.expose_secret().to_owned(),
        Err(err) => panic!("the token endpoint refused the assertion: {err:?}"),
    }
}

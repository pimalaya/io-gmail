//! Offline history poll tests: the watch runs against a scripted
//! server answering one canned response per request, covering what it
//! reports, where it starts from and how it stops.

mod common;

use std::collections::VecDeque;

use io_gmail::{
    coroutine::*,
    v1::history_poll::{
        GMAIL_HISTORY_POLL_MAX_PAGES, GmailHistoryDiff, GmailHistoryPoll, GmailHistoryPollError,
        GmailHistoryPollYield,
    },
};
use io_http::rfc6750::bearer::HttpAuthBearer;

use common::json_response;

/// What the watch does between two exchanges.
#[derive(Debug)]
enum Event {
    Sleep,
    Diff(GmailHistoryDiff),
    Expired(String),
    Complete(GmailHistoryPollError),
    Starved,
}

/// A scripted server: one canned response per request, in order.
struct Script {
    responses: VecDeque<Vec<u8>>,
    requests: Vec<String>,
}

impl Script {
    fn new(responses: impl IntoIterator<Item = Vec<u8>>) -> Self {
        Self {
            responses: responses.into_iter().collect(),
            requests: Vec::new(),
        }
    }

    /// Runs the watch up to its next event, `Starved` once it reads
    /// past the script.
    fn next(&mut self, poll: &mut GmailHistoryPoll) -> Event {
        let mut response: Option<Vec<u8>> = None;

        loop {
            let arg = response.take();

            match poll.resume(arg.as_deref()) {
                GmailCoroutineState::Complete(Err(err)) => return Event::Complete(err),
                GmailCoroutineState::Complete(Ok(never)) => match never {},
                GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsWrite(bytes)) => {
                    self.requests.push(String::from_utf8(bytes).unwrap());
                }
                GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsRead) => {
                    match self.responses.pop_front() {
                        Some(next) => response = Some(next),
                        None => return Event::Starved,
                    }
                }
                GmailCoroutineState::Yielded(GmailHistoryPollYield::WantsSleep(_)) => {
                    return Event::Sleep;
                }
                GmailCoroutineState::Yielded(GmailHistoryPollYield::Diff(diff)) => {
                    return Event::Diff(diff);
                }
                GmailCoroutineState::Yielded(GmailHistoryPollYield::Expired(history_id)) => {
                    return Event::Expired(history_id);
                }
            }
        }
    }
}

fn auth() -> HttpAuthBearer {
    HttpAuthBearer::new("fake-token")
}

fn ok(body: &str) -> Vec<u8> {
    json_response("HTTP/1.1 200 OK", body)
}

fn error(status_line: &str, code: u16) -> Vec<u8> {
    let body = format!(r#"{{"error":{{"code":{code},"message":"scripted"}}}}"#);
    json_response(status_line, &body)
}

fn profile(history_id: &str) -> Vec<u8> {
    ok(&format!(
        r#"{{"emailAddress":"me@example.com","historyId":"{history_id}"}}"#
    ))
}

/// A history page adding the given message ids.
fn added(ids: &[&str], history_id: &str) -> Vec<u8> {
    let records: Vec<_> = ids
        .iter()
        .map(|id| format!(r#"{{"id":"1","messagesAdded":[{{"message":{{"id":"{id}"}}}}]}}"#))
        .collect();

    ok(&format!(
        r#"{{"history":[{}],"historyId":"{history_id}"}}"#,
        records.join(",")
    ))
}

/// An empty history page pointing to the given next page.
fn page(next_page_token: &str) -> Vec<u8> {
    ok(&format!(
        r#"{{"nextPageToken":"{next_page_token}","historyId":"200"}}"#
    ))
}

fn message(id: &str) -> Vec<u8> {
    ok(&format!(r#"{{"id":"{id}","threadId":"{id}"}}"#))
}

#[test]
fn reports_an_expired_cursor() {
    let mut poll = GmailHistoryPoll::new(&auth(), "me", "INBOX").unwrap();
    let mut script = Script::new([
        profile("100"),
        error("HTTP/1.1 404 Not Found", 404),
        profile("300"),
    ]);

    assert!(matches!(script.next(&mut poll), Event::Sleep));

    let event = script.next(&mut poll);
    assert!(
        matches!(&event, Event::Expired(history_id) if history_id == "300"),
        "got: {event:?}"
    );

    assert!(matches!(script.next(&mut poll), Event::Sleep));
    assert_eq!(script.requests.len(), 3);
}

#[test]
fn skips_a_message_gone_since_it_was_listed() {
    let mut poll = GmailHistoryPoll::new(&auth(), "me", "INBOX").unwrap();
    let mut script = Script::new([
        profile("100"),
        added(&["m1", "m2"], "200"),
        error("HTTP/1.1 404 Not Found", 404),
        message("m2"),
    ]);

    assert!(matches!(script.next(&mut poll), Event::Sleep));

    let Event::Diff(diff) = script.next(&mut poll) else {
        panic!("the tick reported no diff");
    };

    assert_eq!(diff.history_id, "200");
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].id, "m2");
}

#[test]
fn stops_on_a_failed_message_fetch() {
    let mut poll = GmailHistoryPoll::new(&auth(), "me", "INBOX").unwrap();
    let mut script = Script::new([
        profile("100"),
        added(&["m1", "m2"], "200"),
        error("HTTP/1.1 429 Too Many Requests", 429),
        message("m2"),
    ]);

    assert!(matches!(script.next(&mut poll), Event::Sleep));

    let event = script.next(&mut poll);
    let Event::Complete(GmailHistoryPollError::Send(err)) = &event else {
        panic!("the watch went on past the failed fetch: {event:?}");
    };

    assert!(err.is_rate_limited());
    assert_eq!(script.requests.len(), 3);
}

#[test]
fn stops_on_a_profile_without_history_id() {
    let mut poll = GmailHistoryPoll::new(&auth(), "me", "INBOX").unwrap();
    let mut script = Script::new([ok(r#"{"emailAddress":"me@example.com"}"#)]);

    let event = script.next(&mut poll);
    assert!(
        matches!(
            event,
            Event::Complete(GmailHistoryPollError::MissingHistoryId)
        ),
        "got: {event:?}"
    );
    assert_eq!(script.requests.len(), 1);
}

#[test]
fn stops_on_a_repeated_page_token() {
    let mut poll = GmailHistoryPoll::new(&auth(), "me", "INBOX").unwrap();
    let mut script = Script::new([profile("100"), page("p1"), page("p1"), page("p1")]);

    assert!(matches!(script.next(&mut poll), Event::Sleep));

    let event = script.next(&mut poll);
    assert!(
        matches!(
            event,
            Event::Complete(GmailHistoryPollError::RepeatedPageToken)
        ),
        "got: {event:?}"
    );
    assert_eq!(script.requests.len(), 3);
}

#[test]
fn stops_past_the_page_limit() {
    let pages = (0..=GMAIL_HISTORY_POLL_MAX_PAGES).map(|n| page(&format!("p{n}")));
    let mut poll = GmailHistoryPoll::new(&auth(), "me", "INBOX").unwrap();
    let mut script = Script::new([profile("100")].into_iter().chain(pages));

    assert!(matches!(script.next(&mut poll), Event::Sleep));

    let event = script.next(&mut poll);
    assert!(
        matches!(event, Event::Complete(GmailHistoryPollError::TooManyPages)),
        "got: {event:?}"
    );
    assert_eq!(script.requests.len(), 1 + GMAIL_HISTORY_POLL_MAX_PAGES);
}

#[test]
fn completes_again_once_stopped() {
    let mut poll = GmailHistoryPoll::new(&auth(), "me", "INBOX").unwrap();
    let mut script = Script::new([error("HTTP/1.1 500 Internal Server Error", 500)]);

    let event = script.next(&mut poll);
    assert!(
        matches!(event, Event::Complete(GmailHistoryPollError::Send(_))),
        "got: {event:?}"
    );

    let event = script.next(&mut poll);
    assert!(
        matches!(event, Event::Complete(GmailHistoryPollError::Completed)),
        "got: {event:?}"
    );
}

#[test]
fn resumes_from_a_stored_cursor() {
    let mut poll = GmailHistoryPoll::resume_from(&auth(), "me", "INBOX", "100").unwrap();
    let mut script = Script::new([added(&[], "200")]);

    let Event::Diff(diff) = script.next(&mut poll) else {
        panic!("the first tick reported no diff");
    };

    assert_eq!(diff.history_id, "200");
    assert_eq!(script.requests.len(), 1);
    assert!(
        script.requests[0].starts_with(
            "GET /gmail/v1/users/me/history?startHistoryId=100&labelId=INBOX HTTP/1.1\r\n"
        ),
        "got: {}",
        script.requests[0]
    );
}

#[test]
fn refuses_to_resume_from_an_empty_cursor() {
    let result = GmailHistoryPoll::resume_from(&auth(), "me", "INBOX", "");

    assert!(matches!(
        result,
        Err(GmailHistoryPollError::MissingHistoryId)
    ));
}

#[test]
fn takes_a_new_token_between_ticks() {
    let mut poll = GmailHistoryPoll::resume_from(&auth(), "me", "INBOX", "100").unwrap();
    let mut script = Script::new([added(&[], "200"), added(&[], "300")]);

    assert!(matches!(script.next(&mut poll), Event::Diff(_)));
    assert!(matches!(script.next(&mut poll), Event::Sleep));

    poll.auth = HttpAuthBearer::new("next-token");

    assert!(matches!(script.next(&mut poll), Event::Diff(_)));

    let request = script.requests[1].to_ascii_lowercase();
    assert!(request.contains("starthistoryid=200"), "got: {request}");
    assert!(
        request.contains("authorization: bearer next-token\r\n"),
        "got: {request}"
    );
}

//! Offline batch tests: the batch coroutine runs against scripted
//! `multipart/mixed` answers shaped like the ones Google sends,
//! covering the request body, the matching of answers to calls and the
//! per-call errors.

mod common;

use io_gmail::v1::{
    batch::{
        GMAIL_BATCH_MAX_REQUESTS, GmailBatch, GmailBatchRequest, GmailBatchResponse,
        GmailBatchResponseError,
    },
    rest::messages::{GmailMessage, GmailMessageFormat},
    send::{GMAIL_API_BASE, GmailSendError},
};
use io_http::rfc6750::bearer::HttpAuthBearer;
use url::Url;

use common::{batch_part, batch_response, http_response, json_response, run};

const BOUNDARY: &str = "batch_7Tq1kZ0yAAQ_ABrG3kyLhb0AAAA";

const HEADERS: [&str; 3] = ["From", "Subject", "Date"];

fn auth() -> HttpAuthBearer {
    HttpAuthBearer::new("fake-token")
}

fn metadata_get(id: &str) -> GmailBatchRequest {
    GmailBatchRequest::message_get(id, "me", id, GmailMessageFormat::Metadata, &HEADERS).unwrap()
}

fn message_json(id: &str, labels: &str) -> String {
    format!(
        r#"{{
  "id": "{id}",
  "threadId": "{id}",
  "labelIds": [{labels}],
  "snippet": "Hello",
  "payload": {{
    "partId": "",
    "mimeType": "multipart/alternative",
    "filename": "",
    "headers": [
      {{ "name": "From", "value": "Alice <alice@example.com>" }},
      {{ "name": "Subject", "value": "Hello {id}" }},
      {{ "name": "Date", "value": "Tue, 6 Oct 2026 09:00:00 +0200" }}
    ]
  }},
  "sizeEstimate": 4321,
  "historyId": "987654",
  "internalDate": "1791270000000"
}}"#
    )
}

fn error_json(code: u16, message: &str, reason: &str, status: &str) -> String {
    format!(
        r#"{{
  "error": {{
    "code": {code},
    "message": "{message}",
    "errors": [
      {{
        "message": "{message}",
        "domain": "global",
        "reason": "{reason}"
      }}
    ],
    "status": "{status}"
  }}
}}"#
    )
}

fn close() -> String {
    format!("--{BOUNDARY}--\r\n")
}

fn run_batch(
    requests: &[GmailBatchRequest],
    body: &str,
) -> (Vec<GmailBatchResponse<GmailMessage>>, String) {
    let mut coroutine = GmailBatch::<GmailMessage>::new(&auth(), requests).unwrap();
    let (ret, written) = run(&mut coroutine, &batch_response(BOUNDARY, body));
    let out = ret.unwrap();
    assert!(out.keep_alive);
    (out.response, String::from_utf8(written).unwrap())
}

#[test]
fn writes_one_http_part_per_call() {
    let requests = [metadata_get("18c2f0a1"), metadata_get("18c2f0a2")];
    let body = [
        batch_part(
            BOUNDARY,
            "18c2f0a1",
            "HTTP/1.1 200 OK",
            &message_json("18c2f0a1", ""),
        ),
        batch_part(
            BOUNDARY,
            "18c2f0a2",
            "HTTP/1.1 200 OK",
            &message_json("18c2f0a2", ""),
        ),
        close(),
    ]
    .concat();
    let (_, written) = run_batch(&requests, &body);

    assert!(
        written.starts_with("POST /batch/gmail/v1 HTTP/1.1\r\n"),
        "got: {written}"
    );

    let lower = written.to_ascii_lowercase();
    assert!(
        lower.contains("host: www.googleapis.com\r\n"),
        "got: {written}"
    );
    assert!(
        lower.contains("authorization: bearer fake-token\r\n"),
        "got: {written}"
    );

    let (head, body) = written.split_once("\r\n\r\n").unwrap();
    let boundary = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-type")
                .then_some(value.trim())
        })
        .and_then(|value| value.strip_prefix("multipart/mixed; boundary="))
        .expect("multipart content type");

    let expected = format!(
        "--{boundary}\r\n\
         Content-Type: application/http\r\n\
         Content-ID: <18c2f0a1>\r\n\
         \r\n\
         GET /gmail/v1/users/me/messages/18c2f0a1?format=METADATA&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date HTTP/1.1\r\n\
         \r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: application/http\r\n\
         Content-ID: <18c2f0a2>\r\n\
         \r\n\
         GET /gmail/v1/users/me/messages/18c2f0a2?format=METADATA&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date HTTP/1.1\r\n\
         \r\n\
         \r\n\
         --{boundary}--\r\n"
    );
    assert_eq!(body, expected);
}

#[test]
fn reads_every_answer() {
    let requests = [metadata_get("a1"), metadata_get("a2"), metadata_get("a3")];
    let body = [
        batch_part(
            BOUNDARY,
            "a1",
            "HTTP/1.1 200 OK",
            &message_json("a1", r#""INBOX","UNREAD""#),
        ),
        batch_part(
            BOUNDARY,
            "a2",
            "HTTP/1.1 200 OK",
            &message_json("a2", r#""SENT""#),
        ),
        batch_part(
            BOUNDARY,
            "a3",
            "HTTP/1.1 200 OK",
            &message_json("a3", r#""INBOX""#),
        ),
        close(),
    ]
    .concat();
    let (responses, _) = run_batch(&requests, &body);

    assert_eq!(responses.len(), 3);

    for (response, id) in responses.iter().zip(["a1", "a2", "a3"]) {
        assert_eq!(response.id, id);
        assert_eq!(response.status, Some(200));

        let message = response.result.as_ref().unwrap();
        assert_eq!(message.id, id);
        assert_eq!(message.size_estimate, Some(4321));

        let payload = message.payload.as_ref().unwrap();
        assert_eq!(
            payload.header("subject"),
            Some(format!("Hello {id}").as_str())
        );
    }

    let message = responses[0].result.as_ref().unwrap();
    assert_eq!(message.label_ids, ["INBOX", "UNREAD"]);
}

#[test]
fn matches_answers_out_of_order() {
    let requests = [metadata_get("b1"), metadata_get("b2"), metadata_get("b3")];
    let body = [
        batch_part(BOUNDARY, "b3", "HTTP/1.1 200 OK", &message_json("b3", "")),
        batch_part(BOUNDARY, "b1", "HTTP/1.1 200 OK", &message_json("b1", "")),
        batch_part(BOUNDARY, "b2", "HTTP/1.1 200 OK", &message_json("b2", "")),
        close(),
    ]
    .concat();
    let (responses, _) = run_batch(&requests, &body);

    let ids: Vec<_> = responses.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["b1", "b2", "b3"]);

    for response in &responses {
        assert_eq!(response.result.as_ref().unwrap().id, response.id);
    }
}

#[test]
fn keeps_each_call_error() {
    let requests = [
        metadata_get("c1"),
        metadata_get("c2"),
        metadata_get("c3"),
        metadata_get("c4"),
        metadata_get("c5"),
        metadata_get("c6"),
    ];
    let body = [
        batch_part(BOUNDARY, "c1", "HTTP/1.1 200 OK", &message_json("c1", "")),
        batch_part(
            BOUNDARY,
            "c2",
            "HTTP/1.1 404 Not Found",
            &error_json(
                404,
                "Requested entity was not found.",
                "notFound",
                "NOT_FOUND",
            ),
        ),
        batch_part(
            BOUNDARY,
            "c3",
            "HTTP/1.1 429 Too Many Requests",
            &error_json(
                429,
                "Too many concurrent requests for user",
                "rateLimitExceeded",
                "RESOURCE_EXHAUSTED",
            ),
        ),
        batch_part(
            BOUNDARY,
            "c4",
            "HTTP/1.1 403 Forbidden",
            &error_json(
                403,
                "Rate Limit Exceeded",
                "rateLimitExceeded",
                "PERMISSION_DENIED",
            ),
        ),
        batch_part(
            BOUNDARY,
            "c5",
            "HTTP/1.1 403 Forbidden",
            &error_json(
                403,
                "Daily Limit Exceeded",
                "dailyLimitExceeded",
                "PERMISSION_DENIED",
            ),
        ),
        // NOTE: no part for c6
        close(),
    ]
    .concat();
    let (responses, _) = run_batch(&requests, &body);

    assert_eq!(responses.len(), 6);
    assert!(responses[0].result.is_ok());

    let err = |i: usize| -> &GmailBatchResponseError { responses[i].result.as_ref().unwrap_err() };

    assert_eq!(responses[1].status, Some(404));
    assert!(err(1).is_not_found());
    assert!(!err(1).is_retryable());
    assert_eq!(err(1).api().unwrap().reasons, ["notFound"]);
    assert_eq!(
        err(1).api().unwrap().message,
        "Requested entity was not found."
    );

    assert_eq!(responses[2].status, Some(429));
    assert!(err(2).is_rate_limited());
    assert!(err(2).is_retryable());
    assert_eq!(
        err(2).api().unwrap().google_status.as_deref(),
        Some("RESOURCE_EXHAUSTED")
    );

    assert_eq!(responses[3].status, Some(403));
    assert!(err(3).is_rate_limited());
    assert!(err(3).is_retryable());
    assert!(err(3).api().unwrap().has_reason("rateLimitExceeded"));

    assert_eq!(responses[4].status, Some(403));
    assert!(!err(4).is_rate_limited());
    assert!(!err(4).is_retryable());

    assert_eq!(responses[5].status, None);
    assert!(matches!(err(5), GmailBatchResponseError::Missing));
    assert!(err(5).is_retryable());
}

#[test]
fn reads_bare_lf_answers() {
    let requests = [metadata_get("d1"), metadata_get("d2")];
    let body = [
        batch_part(BOUNDARY, "d2", "HTTP/1.1 200 OK", &message_json("d2", "")),
        batch_part(
            BOUNDARY,
            "d1",
            "HTTP/1.1 404 Not Found",
            &error_json(
                404,
                "Requested entity was not found.",
                "notFound",
                "NOT_FOUND",
            ),
        ),
        close(),
    ]
    .concat()
    .replace("\r\n", "\n");
    let (responses, _) = run_batch(&requests, &body);

    assert!(responses[0].result.as_ref().unwrap_err().is_not_found());
    assert_eq!(responses[1].result.as_ref().unwrap().id, "d2");
}

#[test]
fn reads_quoted_boundary_and_preamble() {
    let requests = [metadata_get("e1")];
    let body = [
        "This is a preamble.\r\n".to_string(),
        batch_part(BOUNDARY, "e1", "HTTP/1.1 200 OK", &message_json("e1", "")),
        close(),
        "an epilogue\r\n".to_string(),
    ]
    .concat();

    let content_type = format!("multipart/mixed; boundary=\"{BOUNDARY}\"");
    let response = http_response(
        "HTTP/1.1 200 OK",
        &[("Content-Type", &content_type)],
        body.as_bytes(),
    );

    let mut coroutine = GmailBatch::<GmailMessage>::new(&auth(), &requests).unwrap();
    let (ret, _) = run(&mut coroutine, &response);
    let responses = ret.unwrap().response;

    assert_eq!(responses[0].result.as_ref().unwrap().id, "e1");
}

#[test]
fn reads_any_get_call() {
    let url = Url::parse(GMAIL_API_BASE)
        .unwrap()
        .join("users/me/labels/INBOX")
        .unwrap();
    let requests = [GmailBatchRequest::get("label", &url)];
    let body = [
        batch_part(
            BOUNDARY,
            "label",
            "HTTP/1.1 200 OK",
            r#"{"id":"INBOX","name":"INBOX","type":"system","messagesTotal":12}"#,
        ),
        close(),
    ]
    .concat();

    let mut coroutine = GmailBatch::<serde_json::Value>::new(&auth(), &requests).unwrap();
    let (ret, written) = run(&mut coroutine, &batch_response(BOUNDARY, &body));
    let responses = ret.unwrap().response;

    let written = String::from_utf8(written).unwrap();
    assert!(
        written.contains("GET /gmail/v1/users/me/labels/INBOX HTTP/1.1\r\n"),
        "got: {written}"
    );
    assert_eq!(responses[0].result.as_ref().unwrap()["messagesTotal"], 12);
}

#[test]
fn surfaces_outer_errors() {
    let response = json_response(
        "HTTP/1.1 401 Unauthorized",
        &error_json(
            401,
            "Request had invalid authentication credentials.",
            "authError",
            "UNAUTHENTICATED",
        ),
    );
    let mut coroutine = GmailBatch::<GmailMessage>::new(&auth(), &[metadata_get("f1")]).unwrap();
    let (ret, _) = run(&mut coroutine, &response);
    let err = ret.unwrap_err();

    assert_eq!(err.status(), Some(401));
    assert_eq!(err.api().unwrap().reasons, ["authError"]);
}

#[test]
fn refuses_non_multipart_answers() {
    let response = json_response("HTTP/1.1 200 OK", "{}");
    let mut coroutine = GmailBatch::<GmailMessage>::new(&auth(), &[metadata_get("g1")]).unwrap();
    let (ret, _) = run(&mut coroutine, &response);

    assert!(matches!(ret.unwrap_err(), GmailSendError::ParseResponse(_)));
}

#[test]
fn refuses_invalid_batches() {
    let new = |requests: &[GmailBatchRequest]| GmailBatch::<GmailMessage>::new(&auth(), requests);
    let invalid = |result: Result<_, _>| matches!(result, Err(GmailSendError::InvalidRequest(_)));

    assert!(invalid(new(&[])));

    let too_many: Vec<_> = (0..=GMAIL_BATCH_MAX_REQUESTS)
        .map(|i| metadata_get(&format!("m{i}")))
        .collect();
    assert!(invalid(new(&too_many)));
    assert!(new(&too_many[..GMAIL_BATCH_MAX_REQUESTS]).is_ok());

    assert!(invalid(new(&[metadata_get("h1"), metadata_get("h1")])));

    let bad_id = GmailBatchRequest {
        id: "h<1>".into(),
        path: "/gmail/v1/users/me/messages/h1".into(),
    };
    assert!(invalid(new(&[bad_id])));

    let bad_path = GmailBatchRequest {
        id: "h2".into(),
        path: "/gmail/v1/users/me/messages/h2 HTTP/1.1\r\nX-Injected: 1".into(),
    };
    assert!(invalid(new(&[bad_path])));
}

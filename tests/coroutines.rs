//! Offline coroutine tests: every coroutine runs against a scripted
//! in-memory HTTP response, covering request shape, response parsing
//! and error surfacing.

mod common;

use io_gmail::v1::{
    query::{is_false, to_query_pairs},
    rest::{
        history::{
            GmailHistoryType,
            list::{GmailHistoryList, GmailHistoryListParams},
        },
        labels::{
            GmailLabel, GmailLabelListVisibility, create::GmailLabelCreate,
            delete::GmailLabelDelete, list::GmailLabelsList, patch::GmailLabelPatch,
        },
        messages::{
            GmailMessageListVisibility, decode_raw, encode_raw,
            list::{GmailMessagesList, GmailMessagesListParams},
        },
        users::get_profile::GmailProfileGet,
    },
    send::{GmailSendError, parse_api_error},
};
use io_http::rfc6750::bearer::HttpAuthBearer;

use common::{empty_response, json_response, run};

fn auth() -> HttpAuthBearer {
    HttpAuthBearer::new("fake-token")
}

#[test]
fn gets_profile() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{"emailAddress":"me@example.com","messagesTotal":12,"threadsTotal":8}"#,
    );
    let mut coroutine = GmailProfileGet::new(&auth(), "me").unwrap();
    let (ret, _) = run(&mut coroutine, &response);
    let out = ret.unwrap();

    assert_eq!(out.response.email_address, "me@example.com");
    assert_eq!(out.response.messages_total, Some(12));
}

#[test]
fn lists_labels() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{"labels":[{"id":"INBOX","name":"INBOX","type":"system"}]}"#,
    );
    let mut coroutine = GmailLabelsList::new(&auth(), "me").unwrap();
    let (ret, _) = run(&mut coroutine, &response);
    let out = ret.unwrap();

    assert_eq!(out.response.labels.len(), 1);
    assert_eq!(out.response.labels[0].id, "INBOX");
}

#[test]
fn creates_label_with_visibilities() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{"id":"Label_1","name":"todo","type":"user"}"#,
    );
    let label = GmailLabel {
        name: "todo".into(),
        label_list_visibility: Some(GmailLabelListVisibility::LabelShow),
        message_list_visibility: Some(GmailMessageListVisibility::Show),
        ..Default::default()
    };
    let mut coroutine = GmailLabelCreate::new(&auth(), "me", &label).unwrap();
    let (ret, written) = run(&mut coroutine, &response);

    assert_eq!(ret.unwrap().response.id, "Label_1");

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.contains("\"labelListVisibility\":\"labelShow\""),
        "got: {request}"
    );
    assert!(
        request.contains("\"messageListVisibility\":\"show\""),
        "got: {request}"
    );
}

#[test]
fn rejects_empty_label_name() {
    let label = GmailLabel {
        name: "  ".into(),
        ..Default::default()
    };
    let result = GmailLabelCreate::new(&auth(), "me", &label);
    assert!(matches!(result, Err(GmailSendError::InvalidRequest(_))));
}

#[test]
fn updates_label_with_patch() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{"id":"Label_1","name":"renamed","type":"user"}"#,
    );
    let label = GmailLabel {
        id: "Label_1".into(),
        name: "renamed".into(),
        ..Default::default()
    };
    let mut coroutine = GmailLabelPatch::new(&auth(), "me", &label).unwrap();
    let (ret, written) = run(&mut coroutine, &response);

    assert_eq!(ret.unwrap().response.name, "renamed");

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("PATCH /gmail/v1/users/me/labels/Label_1"),
        "got: {request}"
    );
}

#[test]
fn deletes_label() {
    let response = empty_response("HTTP/1.1 204 No Content");
    let mut coroutine = GmailLabelDelete::new(&auth(), "me", "Label_1").unwrap();
    let (ret, written) = run(&mut coroutine, &response);

    ret.unwrap();

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with("DELETE /gmail/v1/users/me/labels/Label_1"),
        "got: {request}"
    );
}

#[test]
fn surfaces_api_errors() {
    let response = json_response(
        "HTTP/1.1 403 Forbidden",
        r#"{"error":{"code":403,"message":"insufficient permissions"}}"#,
    );
    let mut coroutine = GmailLabelsList::new(&auth(), "me").unwrap();
    let (ret, _) = run(&mut coroutine, &response);

    match ret.unwrap_err() {
        GmailSendError::Api(err) => {
            assert_eq!(err.status, 403);
            assert_eq!(err.message, "insufficient permissions");
        }
        err => panic!("unexpected error: {err}"),
    }
}

#[test]
fn parses_error_envelope() {
    let (status, message) =
        parse_api_error(400, br#"{"error":{"code":401,"message":"bad token"}}"#);
    assert_eq!(status, 401);
    assert_eq!(message, "bad token");
}

#[test]
fn falls_back_when_message_missing() {
    let (status, message) = parse_api_error(403, br#"{"error":{"code":403}}"#);
    assert_eq!(status, 403);
    assert_eq!(message, "unknown Gmail API error");
}

#[test]
fn handles_non_json_error_body() {
    let (status, message) = parse_api_error(502, b"upstream failure");
    assert_eq!(status, 502);
    assert_eq!(message, "upstream failure");
}

#[test]
fn round_trips_raw_messages() {
    let raw = b"Subject: test\r\n\r\nhello";
    assert_eq!(decode_raw(&encode_raw(raw)).unwrap(), raw);
}

#[test]
fn decodes_padded_and_wrapped_raw_messages() {
    assert_eq!(decode_raw("SGVsbG8=").unwrap(), b"Hello");
    assert_eq!(decode_raw("SGVs\nbG8=\r\n").unwrap(), b"Hello");
}

#[test]
fn serializes_params_into_query_pairs() {
    #[derive(serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Params<'a> {
        q: Option<&'a str>,
        page_token: Option<&'a str>,
        label_ids: &'a [&'a str],
        max_results: Option<u32>,
        #[serde(skip_serializing_if = "is_false")]
        include_spam_trash: bool,
    }

    let params = Params {
        q: Some("is:unread"),
        page_token: None,
        label_ids: &["INBOX", "UNREAD"],
        max_results: Some(10),
        include_spam_trash: false,
    };

    let pairs = to_query_pairs(&params);

    // NOTE: None and the false flag vanish; the slice expands into
    // one repeated key per element; field names come from the serde
    // rename.
    assert_eq!(
        pairs,
        vec![
            ("q".to_string(), "is:unread".to_string()),
            ("labelIds".to_string(), "INBOX".to_string()),
            ("labelIds".to_string(), "UNREAD".to_string()),
            ("maxResults".to_string(), "10".to_string()),
        ],
    );
}

fn api_error(response: &[u8]) -> GmailSendError {
    let mut coroutine = GmailLabelsList::new(&auth(), "me").unwrap();
    let (ret, _) = run(&mut coroutine, response);
    ret.unwrap_err()
}

#[test]
fn keeps_google_error_codes() {
    let response = json_response(
        "HTTP/1.1 403 Forbidden",
        r#"{
  "error": {
    "code": 403,
    "message": "Quota exceeded for quota metric 'Total Query Cost' and limit 'Units per minute per user' of service 'gmail.googleapis.com' for consumer 'project_number:123456789'.",
    "errors": [
      {
        "message": "Quota exceeded for quota metric 'Total Query Cost' and limit 'Units per minute per user' of service 'gmail.googleapis.com' for consumer 'project_number:123456789'.",
        "domain": "usageLimits",
        "reason": "rateLimitExceeded"
      }
    ],
    "status": "PERMISSION_DENIED",
    "details": [
      {
        "@type": "type.googleapis.com/google.rpc.ErrorInfo",
        "reason": "RATE_LIMIT_EXCEEDED",
        "domain": "googleapis.com",
        "metadata": {
          "quota_limit": "UnitsPerMinutePerUser",
          "service": "gmail.googleapis.com",
          "consumer": "projects/123456789"
        }
      }
    ]
  }
}"#,
    );
    let err = api_error(&response);
    let api = err.api().unwrap();

    assert_eq!(api.status, 403);
    assert_eq!(api.reasons, ["rateLimitExceeded"]);
    assert_eq!(api.google_status.as_deref(), Some("PERMISSION_DENIED"));
    assert_eq!(api.detail_reasons, ["RATE_LIMIT_EXCEEDED"]);
    assert!(err.is_rate_limited());
    assert!(err.is_retryable());
    assert!(!err.is_not_found());
}

#[test]
fn tells_rate_limits_from_daily_quota() {
    let error = |status_line: &str, code: u16, reason: &str, status: &str| {
        api_error(&json_response(
            status_line,
            &format!(
                r#"{{"error":{{"code":{code},"message":"limited","errors":[{{"domain":"usageLimits","reason":"{reason}","message":"limited"}}],"status":"{status}"}}}}"#
            ),
        ))
    };

    let user = error(
        "HTTP/1.1 403 Forbidden",
        403,
        "userRateLimitExceeded",
        "PERMISSION_DENIED",
    );
    assert!(user.is_rate_limited());

    let quota = error(
        "HTTP/1.1 403 Forbidden",
        403,
        "quotaExceeded",
        "PERMISSION_DENIED",
    );
    assert!(quota.is_rate_limited());

    let concurrent = error(
        "HTTP/1.1 429 Too Many Requests",
        429,
        "rateLimitExceeded",
        "RESOURCE_EXHAUSTED",
    );
    assert!(concurrent.is_rate_limited());

    let daily = error(
        "HTTP/1.1 403 Forbidden",
        403,
        "dailyLimitExceeded",
        "PERMISSION_DENIED",
    );
    assert!(daily.api().unwrap().has_reason("dailyLimitExceeded"));
    assert!(!daily.is_rate_limited());
    assert!(!daily.is_retryable());

    let denied = error(
        "HTTP/1.1 403 Forbidden",
        403,
        "insufficientPermissions",
        "PERMISSION_DENIED",
    );
    assert!(!denied.is_rate_limited());

    let details_only = api_error(&json_response(
        "HTTP/1.1 403 Forbidden",
        r#"{"error":{"code":403,"message":"limited","details":[{"@type":"type.googleapis.com/google.rpc.ErrorInfo","reason":"RATE_LIMIT_EXCEEDED"}]}}"#,
    ));
    assert!(details_only.is_rate_limited());

    let unavailable = api_error(&json_response(
        "HTTP/1.1 503 Service Unavailable",
        r#"{"error":{"code":503,"message":"The service is currently unavailable.","status":"UNAVAILABLE"}}"#,
    ));
    assert!(!unavailable.is_rate_limited());
    assert!(unavailable.is_retryable());
}

#[test]
fn lists_account_messages() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{"messages":[{"id":"18c2f0a1","threadId":"18c2f0a0"},{"id":"18c2f09f","threadId":"18c2f09f"}],"nextPageToken":"page-2","resultSizeEstimate":1000}"#,
    );
    let params = GmailMessagesListParams {
        max_results: Some(500),
        page_token: Some("page-1"),
        include_spam_trash: true,
        ..Default::default()
    };
    let mut coroutine = GmailMessagesList::new(&auth(), "me", &params).unwrap();
    let (ret, written) = run(&mut coroutine, &response);
    let out = ret.unwrap().response;

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with(
            "GET /gmail/v1/users/me/messages?maxResults=500&pageToken=page-1&includeSpamTrash=true HTTP/1.1\r\n"
        ),
        "got: {request}"
    );

    assert_eq!(out.messages.len(), 2);
    assert_eq!(out.messages[0].id, "18c2f0a1");
    assert_eq!(out.messages[0].thread_id.as_deref(), Some("18c2f0a0"));
    assert_eq!(out.next_page_token.as_deref(), Some("page-2"));
}

#[test]
fn lists_account_history() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{
  "history": [
    {
      "id": "1235",
      "messages": [{ "id": "m1", "threadId": "t1" }],
      "messagesAdded": [
        { "message": { "id": "m1", "threadId": "t1", "labelIds": ["INBOX", "UNREAD", "CATEGORY_UPDATES"] } }
      ]
    },
    {
      "id": "1236",
      "messages": [{ "id": "m2", "threadId": "t2" }],
      "labelsRemoved": [
        { "message": { "id": "m2", "threadId": "t2", "labelIds": ["IMPORTANT"] }, "labelIds": ["INBOX"] }
      ]
    },
    {
      "id": "1237",
      "messages": [{ "id": "m3", "threadId": "t3" }],
      "labelsAdded": [
        { "message": { "id": "m3", "threadId": "t3", "labelIds": ["INBOX", "STARRED"] }, "labelIds": ["STARRED"] }
      ]
    },
    {
      "id": "1238",
      "messages": [{ "id": "m4", "threadId": "t4" }],
      "messagesDeleted": [
        { "message": { "id": "m4", "threadId": "t4" } }
      ]
    }
  ],
  "nextPageToken": "history-2",
  "historyId": "1240"
}"#,
    );
    let types = [
        GmailHistoryType::MessageAdded,
        GmailHistoryType::MessageDeleted,
        GmailHistoryType::LabelAdded,
        GmailHistoryType::LabelRemoved,
    ];
    let params = GmailHistoryListParams {
        start_history_id: "1234",
        history_types: &types,
        max_results: Some(500),
        ..Default::default()
    };
    let mut coroutine = GmailHistoryList::new(&auth(), "me", &params).unwrap();
    let (ret, written) = run(&mut coroutine, &response);
    let out = ret.unwrap().response;

    let request = String::from_utf8_lossy(&written);
    assert!(
        request.starts_with(
            "GET /gmail/v1/users/me/history?startHistoryId=1234&historyTypes=messageAdded&historyTypes=messageDeleted&historyTypes=labelAdded&historyTypes=labelRemoved&maxResults=500 HTTP/1.1\r\n"
        ),
        "got: {request}"
    );

    assert_eq!(out.history.len(), 4);
    assert_eq!(out.history_id.as_deref(), Some("1240"));
    assert_eq!(out.next_page_token.as_deref(), Some("history-2"));

    let added = &out.history[0].messages_added[0].message;
    assert_eq!(added.id, "m1");
    assert_eq!(added.label_ids, ["INBOX", "UNREAD", "CATEGORY_UPDATES"]);

    let removed = &out.history[1].labels_removed[0];
    assert_eq!(removed.message.id, "m2");
    assert_eq!(removed.message.label_ids, ["IMPORTANT"]);
    assert_eq!(removed.label_ids, ["INBOX"]);

    let labelled = &out.history[2].labels_added[0];
    assert_eq!(labelled.message.id, "m3");
    assert_eq!(labelled.message.label_ids, ["INBOX", "STARRED"]);
    assert_eq!(labelled.label_ids, ["STARRED"]);

    let deleted = &out.history[3].messages_deleted[0].message;
    assert_eq!(deleted.id, "m4");
    assert!(deleted.label_ids.is_empty());
}

#[test]
fn tells_expired_history_cursor() {
    let response = json_response(
        "HTTP/1.1 404 Not Found",
        r#"{"error":{"code":404,"message":"Requested entity was not found.","errors":[{"message":"Requested entity was not found.","domain":"global","reason":"notFound"}],"status":"NOT_FOUND"}}"#,
    );
    let params = GmailHistoryListParams {
        start_history_id: "1",
        ..Default::default()
    };
    let mut coroutine = GmailHistoryList::new(&auth(), "me", &params).unwrap();
    let (ret, _) = run(&mut coroutine, &response);
    let err = ret.unwrap_err();

    assert!(err.is_history_expired());
    assert!(err.is_not_found());
    assert!(!err.is_retryable());
    assert_eq!(
        err.api().unwrap().google_status.as_deref(),
        Some("NOT_FOUND")
    );
}

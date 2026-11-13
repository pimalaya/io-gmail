//! Offline coroutine tests: every coroutine runs against a scripted
//! in-memory HTTP response, covering request shape, response parsing
//! and error surfacing.

mod common;

use std::time::Duration;

use io_gmail::v1::{
    query::{is_false, to_field_pairs, to_query_pairs},
    rest::{
        history::{
            GmailHistoryType,
            list::{GmailHistoryList, GmailHistoryListParams},
        },
        labels::{
            GmailLabel, GmailLabelListVisibility, GmailLabelType, create::GmailLabelCreate,
            delete::GmailLabelDelete, list::GmailLabelsList, patch::GmailLabelPatch,
            update::GmailLabelUpdate,
        },
        messages::{
            GmailMessageListVisibility, decode_raw, encode_raw,
            list::{GmailMessagesList, GmailMessagesListParams},
        },
        settings::{
            GmailAutoForwarding, GmailDisposition, GmailExpungeBehavior, GmailImapSettings,
            GmailPopAccessWindow, GmailPopSettings, GmailVerificationStatus,
            filters::{GmailFilter, GmailFilterSizeComparison},
            forwarding_addresses::GmailForwardingAddress,
            send_as::{GmailSecurityMode, GmailSendAs},
        },
        users::get_profile::GmailProfileGet,
    },
    send::{GmailSendError, parse_api_error},
};
use io_http::rfc6750::bearer::HttpAuthBearer;

use common::{empty_response, http_response, json_response, run};

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
fn refuses_tokens_holding_control_characters() {
    for token in ["fake\n", "fake\r\nX-Injected: 1", "fa\tke", "fa\u{7f}ke"] {
        let auth = HttpAuthBearer::new(token);

        let Err(GmailSendError::InvalidRequest(message)) = GmailProfileGet::new(&auth, "me") else {
            panic!("token {token:?} reached the Authorization header");
        };

        assert!(!message.contains("fa"), "got: {message}");
    }
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
fn summarizes_non_json_error_bodies() {
    let page = format!(
        "<html><head><title>502 Bad Gateway</title></head><body>{}</body></html>",
        "<p>upstream failure</p>".repeat(1000)
    );
    let (_, message) = parse_api_error(502, page.as_bytes());
    assert_eq!(message, "502 Bad Gateway");

    let (_, message) = parse_api_error(502, b"<p>upstream\n  <b>failure</b></p>");
    assert_eq!(message, "upstream failure");

    let (_, message) = parse_api_error(502, "x".repeat(10_000).as_bytes());
    assert_eq!(message, format!("{}…", "x".repeat(200)));
}

#[test]
fn reads_label_values_it_does_not_know() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{"labels":[{"id":"Label_1","name":"todo","type":"shared","messageListVisibility":"dim","labelListVisibility":"labelShowIfStarred"}]}"#,
    );
    let mut coroutine = GmailLabelsList::new(&auth(), "me").unwrap();
    let (ret, _) = run(&mut coroutine, &response);
    let label = &ret.unwrap().response.labels[0];

    assert_eq!(label.id, "Label_1");
    assert_eq!(label.label_type, Some(GmailLabelType::Unknown));
    assert_eq!(
        label.message_list_visibility,
        Some(GmailMessageListVisibility::Unknown)
    );
    assert_eq!(
        label.label_list_visibility,
        Some(GmailLabelListVisibility::Unknown)
    );
}

#[test]
fn reads_setting_values_it_does_not_know() {
    let forwarding = r#"{"enabled":true,"disposition":"snooze"}"#;
    let forwarding: GmailAutoForwarding = serde_json::from_str(forwarding).unwrap();
    assert_eq!(forwarding.disposition, Some(GmailDisposition::Unknown));

    let imap = r#"{"enabled":true,"expungeBehavior":"shred"}"#;
    let imap: GmailImapSettings = serde_json::from_str(imap).unwrap();
    assert_eq!(imap.expunge_behavior, Some(GmailExpungeBehavior::Unknown));

    let pop = r#"{"accessWindow":"lastWeek","disposition":"snooze"}"#;
    let pop: GmailPopSettings = serde_json::from_str(pop).unwrap();
    assert_eq!(pop.access_window, Some(GmailPopAccessWindow::Unknown));
    assert_eq!(pop.disposition, Some(GmailDisposition::Unknown));

    let address = r#"{"forwardingEmail":"a@example.com","verificationStatus":"revoked"}"#;
    let address: GmailForwardingAddress = serde_json::from_str(address).unwrap();
    assert_eq!(
        address.verification_status,
        Some(GmailVerificationStatus::Unknown)
    );

    let send_as = r#"{"sendAsEmail":"a@example.com","smtpMsa":{"host":"smtp.example.com","port":587,"securityMode":"quic"}}"#;
    let send_as: GmailSendAs = serde_json::from_str(send_as).unwrap();
    assert_eq!(
        send_as.smtp_msa.unwrap().security_mode,
        Some(GmailSecurityMode::Unknown)
    );

    let filter = r#"{"id":"f1","criteria":{"size":10,"sizeComparison":"equal"}}"#;
    let filter: GmailFilter = serde_json::from_str(filter).unwrap();
    assert_eq!(
        filter.criteria.unwrap().size_comparison,
        Some(GmailFilterSizeComparison::Unknown)
    );
}

#[test]
fn refuses_to_write_back_a_value_it_does_not_know() {
    let label = GmailLabel {
        id: "Label_1".into(),
        name: "todo".into(),
        label_list_visibility: Some(GmailLabelListVisibility::Unknown),
        ..Default::default()
    };
    let result = GmailLabelUpdate::new(&auth(), "me", &label);

    assert!(matches!(result, Err(GmailSendError::SerializeRequest(_))));
}

#[test]
fn keeps_the_delay_to_retry_after() {
    let status = "HTTP/1.1 429 Too Many Requests";
    let plain = br#"{"error":{"code":429,"message":"slow down"}}"#;
    let info = br#"{"error":{"code":429,"message":"slow down","details":[{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"1.5s"}]}}"#;
    let retry_after = |headers: &[(&str, &str)], body: &[u8]| {
        api_error(&http_response(status, headers, body)).retry_after()
    };

    assert_eq!(retry_after(&[], plain), None);
    assert_eq!(
        retry_after(&[("Retry-After", "120")], plain),
        Some(Duration::from_secs(120))
    );
    assert_eq!(retry_after(&[], info), Some(Duration::from_millis(1500)));
    assert_eq!(
        retry_after(&[("Retry-After", "7")], info),
        Some(Duration::from_secs(7))
    );

    // NOTE: the date form of the header is not read.
    let date = [("Retry-After", "Fri, 13 Nov 2026 10:00:00 GMT")];
    assert_eq!(retry_after(&date, plain), None);
    assert_eq!(retry_after(&date, info), Some(Duration::from_millis(1500)));
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
fn refuses_raw_messages_outside_base64url() {
    // NOTE: `+` and `/` belong to the standard alphabet, not base64url.
    assert!(decode_raw("Pz8+Pz8/").is_err());
    assert!(decode_raw("SGVs!bG8").is_err());
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

    let pairs = to_query_pairs(&params).unwrap();

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

#[test]
fn refuses_params_it_cannot_flatten() {
    #[derive(serde::Serialize)]
    struct Nested {
        q: &'static str,
    }

    #[derive(serde::Serialize)]
    struct Params {
        q: &'static str,
        nested: Nested,
    }

    let params = Params {
        q: "is:unread",
        nested: Nested { q: "is:read" },
    };

    assert!(to_query_pairs(&params).is_err());
    assert!(to_query_pairs("is:unread").is_err());
    assert!(to_field_pairs("q", &params).is_err());
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

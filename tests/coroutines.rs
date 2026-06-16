mod common;

use io_gmail::v1::{
    rest::{
        get_profile::GmailProfileGet,
        labels::{
            GmailLabel, GmailLabelListVisibility, create::GmailLabelCreate,
            delete::GmailLabelDelete, list::GmailLabelsList, patch::GmailLabelPatch,
        },
        messages::{GmailMessageListVisibility, decode_raw, encode_raw},
    },
    send::{GmailSendError, parse_api_error},
};
use secrecy::SecretString;

use common::{drive, empty_response, json_response};

fn auth() -> SecretString {
    SecretString::from("Bearer fake-token".to_string())
}

#[test]
fn gets_profile() {
    let response = json_response(
        "HTTP/1.1 200 OK",
        r#"{"emailAddress":"me@example.com","messagesTotal":12,"threadsTotal":8}"#,
    );
    let mut coroutine = GmailProfileGet::new(&auth(), "me").unwrap();
    let (ret, _) = drive(&mut coroutine, &response);
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
    let (ret, _) = drive(&mut coroutine, &response);
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
    let (ret, written) = drive(&mut coroutine, &response);

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
    let (ret, written) = drive(&mut coroutine, &response);

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
    let (ret, written) = drive(&mut coroutine, &response);

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
    let (ret, _) = drive(&mut coroutine, &response);

    match ret.unwrap_err() {
        GmailSendError::Api { status, message } => {
            assert_eq!(status, 403);
            assert_eq!(message, "insufficient permissions");
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

//! Shared offline test helpers: a scripted coroutine loop and canned
//! HTTP response builders, so no test needs a network.

#![allow(dead_code)]

use io_gmail::coroutine::*;

/// Runs the coroutine to completion against one canned response,
/// returning its terminal value and everything it wrote.
pub fn run<C: GmailCoroutine<Yield = GmailYield>>(
    coroutine: &mut C,
    response: &[u8],
) -> (C::Return, Vec<u8>) {
    let mut written = Vec::new();
    let mut fed = false;
    let mut arg: Option<&[u8]> = None;

    loop {
        match coroutine.resume(arg.take()) {
            GmailCoroutineState::Complete(ret) => return (ret, written),
            GmailCoroutineState::Yielded(GmailYield::WantsWrite(bytes)) => {
                written.extend_from_slice(&bytes);
            }
            GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                if fed {
                    arg = Some(&[]);
                } else {
                    fed = true;
                    arg = Some(response);
                }
            }
        }
    }
}

/// Builds a keep-alive HTTP response carrying a JSON body.
pub fn json_response(status_line: &str, body: &str) -> Vec<u8> {
    http_response(
        status_line,
        &[
            ("Connection", "keep-alive"),
            ("Content-Type", "application/json"),
        ],
        body.as_bytes(),
    )
}

/// Builds a bodiless HTTP response closing the connection.
pub fn empty_response(status_line: &str) -> Vec<u8> {
    http_response(status_line, &[("Connection", "close")], &[])
}

/// Builds an HTTP response with the given headers and body.
pub fn http_response(status_line: &str, headers: &[(&str, &str)], body: &[u8]) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(status_line.as_bytes());
    response.extend_from_slice(b"\r\n");

    for (name, value) in headers {
        response.extend_from_slice(name.as_bytes());
        response.extend_from_slice(b": ");
        response.extend_from_slice(value.as_bytes());
        response.extend_from_slice(b"\r\n");
    }

    response.extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
    response.extend_from_slice(body);
    response
}

/// Builds a keep-alive Gmail batch answer: a `multipart/mixed` body
/// under the given boundary.
pub fn batch_response(boundary: &str, body: &str) -> Vec<u8> {
    let content_type = format!("multipart/mixed; boundary={boundary}");
    http_response(
        "HTTP/1.1 200 OK",
        &[
            ("Connection", "keep-alive"),
            ("Content-Type", &content_type),
        ],
        body.as_bytes(),
    )
}

/// Builds one part of a Gmail batch answer, the way Google writes it:
/// the part headers, then the inner HTTP response with a JSON body.
pub fn batch_part(boundary: &str, id: &str, status_line: &str, json: &str) -> String {
    format!(
        "--{boundary}\r\n\
         Content-Type: application/http\r\n\
         Content-ID: <response-{id}>\r\n\
         \r\n\
         {status_line}\r\n\
         Content-Type: application/json; charset=UTF-8\r\n\
         Vary: Origin\r\n\
         Vary: X-Origin\r\n\
         Vary: Referer\r\n\
         \r\n\
         {json}\r\n"
    )
}

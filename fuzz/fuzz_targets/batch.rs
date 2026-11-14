#![no_main]

//! Coverage-guided fuzz target for the batch answer parser. The oracle:
//! whatever multipart body a server sends, under whatever boundary, the
//! batch coroutine answers every call exactly once, in the order of the
//! calls, never panics and never asks for a read after EOF.

use arbitrary::Arbitrary;
use io_gmail::{
    coroutine::*,
    v1::batch::{GmailBatch, GmailBatchRequest, GmailBatchResponseError},
};
use io_http::rfc6750::bearer::HttpAuthBearer;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

/// How many calls a batch holds at most, and how many ids a part names.
const MAX_REQUESTS: u8 = 8;

#[derive(Arbitrary, Debug)]
struct Input {
    requests: u8,
    boundary: Vec<u8>,
    body: Vec<Token>,
}

/// One piece of the answered body: the building blocks of a multipart
/// document, so the fuzzer reaches past the delimiter scan, or raw bytes.
#[derive(Arbitrary, Debug)]
enum Token {
    Bytes(Vec<u8>),
    Delimiter,
    Close,
    ContentId(u8),
    Status(u16),
    Crlf,
    Lf,
}

fuzz_target!(|input: Input| {
    let requests: Vec<_> = (0..1 + input.requests % MAX_REQUESTS)
        .map(|i| GmailBatchRequest {
            id: format!("{i}"),
            path: format!("/gmail/v1/users/me/messages/{i}"),
            ..Default::default()
        })
        .collect();

    let boundary: String = input
        .boundary
        .iter()
        .filter(|b| b.is_ascii_graphic())
        .map(|b| char::from(*b))
        .collect();

    let mut body = Vec::new();

    for token in &input.body {
        match token {
            Token::Bytes(bytes) => body.extend_from_slice(bytes),
            Token::Delimiter => body.extend_from_slice(format!("--{boundary}").as_bytes()),
            Token::Close => body.extend_from_slice(format!("--{boundary}--").as_bytes()),
            Token::ContentId(i) => {
                let id = i % (MAX_REQUESTS + 1);
                body.extend_from_slice(format!("Content-ID: <response-{id}>").as_bytes());
            }
            Token::Status(status) => {
                body.extend_from_slice(format!("HTTP/1.1 {status} X").as_bytes());
            }
            Token::Crlf => body.extend_from_slice(b"\r\n"),
            Token::Lf => body.push(b'\n'),
        }
    }

    let head = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: multipart/mixed; boundary={boundary}\r\n\
         Content-Length: {}\r\n\
         \r\n",
        body.len()
    );
    let response = [head.as_bytes(), &body].concat();

    let auth = HttpAuthBearer::new("token");
    let mut batch = GmailBatch::<Value>::new(&auth, &requests).unwrap();
    let mut reads = [response.as_slice(), b""].into_iter();
    let mut arg: Option<&[u8]> = None;

    let out = loop {
        match batch.resume(arg.take()) {
            GmailCoroutineState::Yielded(GmailYield::WantsWrite(_)) => {}
            GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                arg = Some(reads.next().expect("asks for a read after EOF"));
            }
            GmailCoroutineState::Complete(Ok(out)) => break out,
            GmailCoroutineState::Complete(Err(_)) => return,
        }
    };

    assert_eq!(out.response.len(), requests.len());

    for (request, response) in requests.iter().zip(&out.response) {
        let missing = matches!(response.result, Err(GmailBatchResponseError::Missing));

        assert_eq!(response.id, request.id);
        assert_eq!(response.status.is_none(), missing);
    }
});

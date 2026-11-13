//! Offline memory test: a counting allocator measures the peak of live
//! bytes while a coroutine reads a large response. It is alone in its
//! binary so that no other test allocates meanwhile.

mod common;

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

use io_gmail::{
    coroutine::*,
    v1::rest::messages::{GmailMessageFormat, get::GmailMessageGet},
};
use io_http::rfc6750::bearer::HttpAuthBearer;

use common::json_response;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

/// The system allocator, counting the live bytes and their peak.
struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let live = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
        PEAK.fetch_max(live, Ordering::Relaxed);
        // SAFETY: the caller upholds the contract of `GlobalAlloc::alloc`.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: the caller upholds the contract of `GlobalAlloc::dealloc`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Size of a read of the std client, and of a TLS record.
const READ_LEN: usize = 16 * 1024;

#[test]
fn reads_a_raw_message_within_twice_its_size() {
    const RAW_LEN: usize = 4 * 1024 * 1024;

    let body = format!(r#"{{"id":"m1","raw":"{}"}}"#, "A".repeat(RAW_LEN));
    let response = json_response("HTTP/1.1 200 OK", &body);
    drop(body);

    let auth = HttpAuthBearer::new("fake-token");

    // NOTE: read by read as a stream gives it, then in one piece.
    for read_len in [READ_LEN, response.len()] {
        let mut coroutine =
            GmailMessageGet::new(&auth, "me", "m1", GmailMessageFormat::Raw, &[]).unwrap();

        let before = LIVE.load(Ordering::Relaxed);
        PEAK.store(before, Ordering::Relaxed);

        let ret = read(&mut coroutine, &response, read_len);
        let peak = PEAK.load(Ordering::Relaxed) - before;

        assert_eq!(ret.unwrap().response.raw.unwrap().len(), RAW_LEN);

        // NOTE: twice is the body beside the message parsed from it.
        assert!(
            peak < RAW_LEN * 2 + 64 * 1024,
            "peak of {peak} bytes for a raw message of {RAW_LEN} read by {read_len}"
        );
    }
}

/// Runs the coroutine to completion, feeding it the response by reads
/// of `read_len` bytes at most.
fn read<C: GmailCoroutine<Yield = GmailYield>>(
    coroutine: &mut C,
    response: &[u8],
    read_len: usize,
) -> C::Return {
    let mut reads = response.chunks(read_len);
    let mut arg: Option<&[u8]> = None;

    loop {
        match coroutine.resume(arg.take()) {
            GmailCoroutineState::Complete(ret) => return ret,
            GmailCoroutineState::Yielded(GmailYield::WantsWrite(_)) => {}
            GmailCoroutineState::Yielded(GmailYield::WantsRead) => {
                arg = Some(reads.next().unwrap_or(&[]));
            }
        }
    }
}

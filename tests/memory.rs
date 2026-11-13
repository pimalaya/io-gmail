//! Offline memory test: a counting allocator measures the peak of live
//! bytes while a coroutine reads a large response. It is alone in its
//! binary so that no other test allocates meanwhile.

mod common;

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

use io_gmail::v1::rest::messages::{GmailMessageFormat, get::GmailMessageGet};
use io_http::rfc6750::bearer::HttpAuthBearer;

use common::{json_response, run};

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

#[test]
fn reads_a_raw_message_within_twice_its_size() {
    const RAW_LEN: usize = 4 * 1024 * 1024;

    let body = format!(r#"{{"id":"m1","raw":"{}"}}"#, "A".repeat(RAW_LEN));
    let response = json_response("HTTP/1.1 200 OK", &body);
    drop(body);

    let auth = HttpAuthBearer::new("fake-token");
    let mut coroutine =
        GmailMessageGet::new(&auth, "me", "m1", GmailMessageFormat::Raw, &[]).unwrap();

    let before = LIVE.load(Ordering::Relaxed);
    PEAK.store(before, Ordering::Relaxed);

    let (ret, _) = run(&mut coroutine, &response);
    let peak = PEAK.load(Ordering::Relaxed) - before;

    assert_eq!(ret.unwrap().response.raw.unwrap().len(), RAW_LEN);

    // NOTE: twice is the receive buffer beside the body it is copied
    // to, then the body beside the message parsed from it.
    assert!(
        peak < RAW_LEN * 5 / 2,
        "peak of {peak} bytes for a raw message of {RAW_LEN}"
    );
}

---
cairn: tasks
change: history-poll-reports-resumes-stops
---

# Tasks

- [x] Write an offline harness for the watch: a scripted server answering one response per request (tests/history_poll.rs)
- [x] Reproduce: six failing tests (silent expiry, a throttled fetch skipped, a profile without history id, a repeated page token, endless pages, a panic on resume after an error), and the missing `resume_from` and `auth` as compile errors
- [x] Yield `GmailHistoryPollYield::Expired` after re-baselining
- [x] Complete on a failed message fetch, skip only a 404
- [x] Add `GmailHistoryPoll::resume_from`, make `auth` public, build the first listing of a tick after the sleep
- [x] Bound the pages of a tick, stop on a repeated page token
- [x] Refuse a missing or empty cursor
- [x] Complete again with `Completed` once stopped
- [x] Follow in the live test (one arm for `Expired`), the crate header and the README
- [x] CHANGELOG, spec, log

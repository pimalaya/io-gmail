---
cairn: log
change: history-poll-reports-resumes-stops
landed: 2026-11-13
---

# The history watcher reports an expired cursor, resumes from a stored one and stops on errors

Four findings of the family audit of 2026-11-13 (GMAIL-01, GMAIL-02, GMAIL-03, GMAIL-11) on `GmailHistoryPoll`, which had no offline test: tests/history_poll.rs now runs it against a scripted server, and each defect failed there first.

The watch used to answer a 404 on `users.history.list` by taking the current cursor and sleeping, and to answer any failed message fetch by leaving the message out of the diff while moving its cursor past it. Both lost changes without a trace on the caller's side. It now yields `Expired` with the new cursor, taken before the event so that the caller's full resync overlaps what follows, and it completes with the error of a fetch that is not a 404. Completing was weighed for the expiry too and set aside: the watch can go on, and only it can take the new cursor before telling.

`resume_from` starts from the cursor of the last diff and runs a tick at once. `auth` is a public field rather than a setter, the family's way for options of a coroutine, and the listing of a tick is built when the watch is resumed after its sleep, where it was built before the sleep with the token of that moment. A watch therefore outlives its token in two ways: the caller replaces `auth` during a sleep, or lets the 401 complete the watch and builds another with `resume_from`.

A tick is bounded to `GMAIL_HISTORY_POLL_MAX_PAGES` pages. 1000 is a choice, not a measure: at the default page size it is 100000 history records, beyond which a full resync is the cheaper way, and the caller learns it through `TooManyPages`. Only a token repeated from one page to the next is caught as `RepeatedPageToken`; a longer loop runs into the page limit. A profile without `historyId`, an empty one, and an empty cursor given to `resume_from` fail with `MissingHistoryId`; an empty `historyId` in a listing leaves the cursor where it was.

Resuming a completed watch returns `Completed` where it panicked. The other coroutines of the family panic there by design; this one differs because its only completion is an error, which a loop written for an infinite watch is likely to resume past.

Not done here: the watch still ignores `keep_alive` from its exchanges, the poll interval is still fixed at 30 seconds, and a message that fails to parse on every fetch stops every watch resumed from the same cursor, which is the caller's to decide.

Capabilities moved: history-poll (new spec file, six requirements).

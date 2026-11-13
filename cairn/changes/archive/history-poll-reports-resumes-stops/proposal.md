---
cairn: change
id: history-poll-reports-resumes-stops
status: landed
created: 2026-11-13
---

# The history watcher reports, resumes and stops

## Why

Four findings of the family audit of 2026-11-13 on `GmailHistoryPoll`, a published coroutine no application of the family runs yet. Together they made it unfit for a caller keeping a replica.

- GMAIL-01: on a 404 from `users.history.list` (the cursor left the retained window) the watch took the current cursor from the profile and went back to sleep. Nothing told the caller, who could not run the full sync that this answer calls for: every change between the two cursors was lost for good.
- GMAIL-02: any error on the fetch of an added message (429, 401, 5xx, a parse error) was skipped at trace level, and the tick still ended with a diff and the listing's cursor. The message was never reported. The skip was meant for one case, a message deleted since its listing.
- GMAIL-03: `new` was the only constructor and always started from the profile, although `GmailHistoryDiff::history_id` is documented as the cursor to persist. The bearer was cloned once, with no way to replace it, so a watch died with its access token and its successor started from now, skipping what happened in between.
- GMAIL-11: the listing followed `nextPageToken` without a page limit or a check for a repeated token; a profile without `historyId` led to a listing with an empty `startHistoryId`, failing as a 400 far from its cause; and resuming after an error hit `unreachable!` under a comment saying the state could not be reached.

## What

- Expiry is an event. On the 404 the watch takes the new cursor first, then yields `GmailHistoryPollYield::Expired(history_id)`, then sleeps. Taking the cursor before the event means a full resync run on receiving it overlaps the next diffs and leaves no gap; the other order would leave one. A typed completion was the other option: it would end a documented infinite watch on a condition it can recover from, and leave the same ordering to every caller.
- A failed message fetch completes the watch with its error, unless it is a 404. No diff is yielded for the tick, so the cursor of the last diff still covers it.
- `GmailHistoryPoll::resume_from(auth, user_id, mailbox, history_id)` starts from a stored cursor and runs its first tick at once. `auth` becomes a public field, read each time a request is built, and the first listing of a tick is now built after the sleep, so a token replaced during the sleep is the one sent.
- A tick reads at most `GMAIL_HISTORY_POLL_MAX_PAGES` (1000) pages and stops on a page token repeated from one page to the next (`TooManyPages`, `RepeatedPageToken`). A missing or empty cursor fails with `MissingHistoryId`, from the profile or from `resume_from`. A completed watch completes again with `Completed`.

## Breaks

`GmailHistoryPollYield` and `GmailHistoryPollError` gain variants, which breaks an exhaustive match on either. No signature changes.

---
cairn: delta
change: history-poll-reports-resumes-stops
---

## ADDED Requirements

All in the new capability history-poll.

### Requirement: The watch starts from the profile or from a stored cursor
`GmailHistoryPoll::new` SHALL take its history cursor from `users.getProfile`, then sleep before its first tick. `GmailHistoryPoll::resume_from` SHALL take the cursor the caller stored and run its first tick at once, without a profile request and without sleeping. Both SHALL fail with `GmailHistoryPollError::MissingHistoryId` when the cursor is missing or empty, and no `users.history.list` request SHALL carry an empty `startHistoryId`.

### Requirement: The token can be replaced between ticks
Every request SHALL carry the value `GmailHistoryPoll::auth` holds when the request is built, and the first listing of a tick SHALL be built after the sleep, when the watch is resumed.

### Requirement: An expired cursor is reported
When `users.history.list` answers 404, the watch SHALL take the current cursor from `users.getProfile`, then yield `GmailHistoryPollYield::Expired` carrying it, then sleep. It SHALL NOT start over without that event. The new cursor is taken before the event so that a full resync run by the caller on receiving it overlaps the next diffs, leaving no gap.

### Requirement: A tick is reported whole or not at all
A message added in a tick and gone when fetched (404) SHALL be left out of the diff. Any other error on a message fetch, a listing or a profile SHALL complete the watch with that error, without a diff for the tick, so that the cursor of the last diff still covers what was not reported.

### Requirement: The listing of a tick is bounded
A tick SHALL read at most `GMAIL_HISTORY_POLL_MAX_PAGES` pages, completing with `GmailHistoryPollError::TooManyPages` when the last of them names a next page, and SHALL complete with `GmailHistoryPollError::RepeatedPageToken` when a page names as next the token it was asked with.

### Requirement: A completed watch stays completed
Once the watch completed with an error, every further resume SHALL complete with `GmailHistoryPollError::Completed`, and SHALL NOT panic.

## MODIFIED Requirements

## REMOVED Requirements

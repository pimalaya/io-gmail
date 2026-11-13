---
cairn: spec
capability: history-poll
status: current
---

# History poll

How `GmailHistoryPoll` watches a mailbox: where it starts, what it reports and how it stops.

### Requirement: The watch starts from the profile or from a stored cursor
`GmailHistoryPoll::new` SHALL take its history cursor from `users.getProfile`, then sleep before its first tick. `GmailHistoryPoll::resume_from` SHALL take the cursor the caller stored and run its first tick at once, without a profile request and without sleeping. Both SHALL fail with `GmailHistoryPollError::MissingHistoryId` when the cursor is missing or empty, and no `users.history.list` request SHALL carry an empty `startHistoryId`.

#### Scenario: A cursor stored from the last diff
- GIVEN the cursor `100`
- WHEN the watch is built by `resume_from` and resumed
- THEN its first request is `users.history.list` with `startHistoryId=100`, and its first event is a diff

#### Scenario: A profile without history id
- GIVEN a profile answer carrying no `historyId`
- WHEN the watch built by `new` reads it
- THEN it completes with `MissingHistoryId` and sends no listing

### Requirement: The token can be replaced between ticks
Every request SHALL carry the value `GmailHistoryPoll::auth` holds when the request is built, and the first listing of a tick SHALL be built after the sleep, when the watch is resumed.

#### Scenario: A token replaced during the sleep
- GIVEN a watch yielding `WantsSleep`
- WHEN the caller replaces `auth`, then resumes
- THEN the listing of the next tick carries the new token

### Requirement: An expired cursor is reported
When `users.history.list` answers 404, the watch SHALL take the current cursor from `users.getProfile`, then yield `GmailHistoryPollYield::Expired` carrying it, then sleep. It SHALL NOT start over without that event. The new cursor is taken before the event so that a full resync run by the caller on receiving it overlaps the next diffs, leaving no gap.

#### Scenario: A cursor out of the retained window
- GIVEN a watch whose listing answers 404, then a profile whose history id is `300`
- WHEN the watch is resumed
- THEN it yields `Expired("300")`, then `WantsSleep`

### Requirement: A tick is reported whole or not at all
A message added in a tick and gone when fetched (404) SHALL be left out of the diff. Any other error on a message fetch, a listing or a profile SHALL complete the watch with that error, without a diff for the tick, so that the cursor of the last diff still covers what was not reported.

#### Scenario: A throttled message fetch
- GIVEN a tick listing two added messages, the fetch of the first answering 429
- WHEN the watch is resumed
- THEN it completes with the rate limit error, yields no diff and does not fetch the second

### Requirement: The listing of a tick is bounded
A tick SHALL read at most `GMAIL_HISTORY_POLL_MAX_PAGES` pages, completing with `GmailHistoryPollError::TooManyPages` when the last of them names a next page, and SHALL complete with `GmailHistoryPollError::RepeatedPageToken` when a page names as next the token it was asked with.

### Requirement: A completed watch stays completed
Once the watch completed with an error, every further resume SHALL complete with `GmailHistoryPollError::Completed`, and SHALL NOT panic.

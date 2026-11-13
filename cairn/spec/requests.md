---
cairn: spec
capability: requests
status: current
---

# Requests

How the coroutines build the Gmail REST requests they send.

### Requirement: An id is one path segment
Every request URL SHALL be built under `GMAIL_API_BASE` by `gmail_api_url`, which percent-encodes each user id, resource id, label id, attachment id and email address as a single path segment. A value holding `/`, `?`, `#` or `..` among other characters SHALL reach the resource it names verbatim, never another path, a query or a fragment. A value that is empty, `.` or `..` names no resource: `gmail_api_url` SHALL refuse it with `GmailSendError::InvalidRequest`.

#### Scenario: A label id carrying a query
- GIVEN the label id `Label_1?x=1`
- WHEN `GmailLabelGet` is built for it
- THEN its URL ends with `/labels/Label_1%3Fx=1` and carries no query

#### Scenario: A label id of two dots
- GIVEN the label id `..`
- WHEN `gmail_api_url` is called with the segments `users`, `me`, `labels` and that id
- THEN it fails with `InvalidRequest`, where the URL of the label collection was built before

### Requirement: The token is one header value
Every coroutine SHALL refuse a bearer token holding a control character (CR, LF, tab and DEL included) with `GmailSendError::InvalidRequest`, when it is built and before any byte is written. The error message SHALL NOT quote the token.

#### Scenario: A token read with its trailing line feed
- GIVEN a token ending with a line feed
- WHEN `GmailProfileGet` is built with it
- THEN it fails with `InvalidRequest` and no request is written

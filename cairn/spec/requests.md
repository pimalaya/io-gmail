---
cairn: spec
capability: requests
status: current
---

# Requests

How the coroutines build the Gmail REST requests they send.

### Requirement: An id is one path segment
Every request URL SHALL be built under `GMAIL_API_BASE` by `gmail_api_url`, which percent-encodes each user id, resource id, label id, attachment id and email address as a single path segment. A value holding `/`, `?`, `#` or `..` SHALL reach the resource it names verbatim, never another path, a query or a fragment.

#### Scenario: A label id carrying a query
- GIVEN the label id `Label_1?x=1`
- WHEN `GmailLabelGet` is built for it
- THEN its URL ends with `/labels/Label_1%3Fx=1` and carries no query

---
cairn: log
change: ids-as-path-segments
landed: 2026-11-12
---

# Ids are percent-encoded path segments

Every coroutine joined `format!("users/{user_id}/labels/{id}")` onto the API base, so an id holding `?` or `#` cut the path (`labels.get` of `Label_375?x=1` returned `Label_375`) and one holding `/` or `..` reached another resource; a live himalaya test found it through an unresolved mailbox name. The new `gmail_api_url` builds each URL from segments, percent-encoding each one, and all 63 request builders use it.

Capabilities moved: requests (new spec file, the requirement "An id is one path segment").

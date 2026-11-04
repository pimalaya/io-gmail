---
cairn: log
change: cairn-adoption
landed: 2026-11-04
---

# Adopt Cairn, retiring docs/

docs/ held a README announcing future documents and no document, so nothing in the repository stated current behaviour. This repository now keeps cairn/spec/ (current truth, one file per capability), cairn/changes/ (in-flight proposals) and cairn/log/ (dated history), with the cairn/ directory as the root marker, AGENTS.md (plus CLAUDE.md) as the activation stanza and cairn/verify.sh, vendored from io-jmap, as the optional checker. docs/ was deleted and CONTRIBUTING.md now points at cairn/.

The spec starts empty and grows by accretion: a capability gets a file the first time a change touches it. History before this entry lives in CHANGELOG.md and the git log, up to the dependency bump (io-http 0.6, Rust 1.89) and the switch from base64 to base64-codec, both of 2026-11-04.

Capabilities moved: none. This change moved documentation, not behaviour.

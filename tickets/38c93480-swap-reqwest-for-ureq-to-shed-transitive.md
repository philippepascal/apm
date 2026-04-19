+++
id = "38c93480"
title = "Swap reqwest for ureq to shed transitive dependencies"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/38c93480-swap-reqwest-for-ureq-to-shed-transitive"
created_at = "2026-04-19T01:24:03.141769Z"
updated_at = "2026-04-19T01:52:17.764037Z"
epic = "7bc3561c"
target_branch = "epic/7bc3561c-trim-dependency-footprint"
+++

## Spec

### Problem

reqwest is one of the largest contributors to the workspace's transitive dependency footprint (pulling in hyper, tokio-native-tls, mio, and their chains — roughly 200 crates). Only five call-sites use it, and every one constructs a plain reqwest::blocking::Client::new() and issues a single HTTP request: apm/src/cmd/register.rs, apm/src/cmd/sessions.rs, apm/src/cmd/revoke.rs, and apm-core/src/github.rs (two calls). No streaming, multipart, cookie-jar, or async features are in use. ureq covers the same blocking request/response shape with a minimal transitive graph. Swapping the five call-sites is the largest single dependency reduction available in this epic.

### Acceptance criteria

Checkboxes; each one independently testable.

### Out of scope

Explicit list of what this ticket does not cover.

### Approach

How the implementation will work.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-19T01:24Z | — | new | philippepascal |
| 2026-04-19T01:47Z | new | groomed | philippepascal |
| 2026-04-19T01:52Z | groomed | in_design | philippepascal |
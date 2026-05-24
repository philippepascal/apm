+++
id = "4691685e"
title = "support for worker_profile manifest"
state = "new"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/4691685e-support-for-worker-profile-manifest"
created_at = "2026-05-24T19:18:32.809526Z"
updated_at = "2026-05-24T19:18:32.809526Z"
+++

## Spec

### Problem

Add support for manifest files for worker profiles. These files will be located in .apm/agents/[agent]/ and be called [worker_profile].toml. They will live along side apm.[worker_profile].toml.

They are optional.

If present, apm start, work, UI dispatcher will use them to look for options and configurations of the agent being spawned. 

First use case will be to have claude spec-writer use opus, and claude worker use sonnet.

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
| 2026-05-24T19:18Z | — | new | philippepascal |

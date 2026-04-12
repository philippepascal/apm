+++
id = "c9a5a1de"
title = "Add version command and version display in UI"
state = "in_design"
priority = 0
effort = 0
risk = 0
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c9a5a1de-add-version-command-and-version-display-"
created_at = "2026-04-12T08:46:45.537269Z"
updated_at = "2026-04-12T08:50:06.063491Z"
+++

## Spec

### Problem

There is no way to check which version of apm is running or whether it's a development or release build. This matters for debugging, bug reports, and confirming deployments.

The version should be available both from the CLI (`apm version` or `apm -v`) and from the UI (displayed when clicking the "Supervisor" title in the supervisor panel).

### Acceptance criteria

- [ ] `apm version` prints the version string to stdout and exits 0
- [ ] The version string includes the semver version matching `apm/Cargo.toml` (e.g. `apm 0.1.3`)
- [ ] The version string includes a build type label: `dev` for debug builds, `release` for release builds
- [ ] `apm --version` (Clap built-in `-V`) also prints the version
- [ ] `GET /api/version` returns `{"version":"<semver>","build":"<dev|release>"}` with HTTP 200
- [ ] The "Supervisor" title span in the UI is clickable (cursor changes to pointer)
- [ ] Clicking the title toggles a version badge inline next to the title (e.g. `Supervisor · v0.1.3 (release)`)
- [ ] The version displayed in the UI matches what `GET /api/version` returns
- [ ] Clicking the title again hides the badge (toggle behaviour)

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
| 2026-04-12T08:46Z | — | new | philippepascal |
| 2026-04-12T08:49Z | new | groomed | apm |
| 2026-04-12T08:50Z | groomed | in_design | philippepascal |
+++
id = "c9a5a1de"
title = "Add version command and version display in UI"
state = "in_progress"
priority = 0
effort = 3
risk = 1
author = "philippepascal"
owner = "philippepascal"
branch = "ticket/c9a5a1de-add-version-command-and-version-display-"
created_at = "2026-04-12T08:46:45.537269Z"
updated_at = "2026-04-12T09:00:33.566607Z"
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

- Automatic version bumping (already handled by `scripts/release.sh`)
- Embedding git commit SHA or dirty-tree status in the version string
- Version compatibility checks between CLI and server
- Changelog or release notes display
- Versioning the `apm-ui` package.json (it tracks `0.0.0` by convention)

### Approach

#### 1. CLI — `apm version` subcommand (`apm/src/`)

**`apm/src/cmd/version.rs`** — new file:
```rust
pub fn run() {
    let version = env!("CARGO_PKG_VERSION");
    let build = if cfg!(debug_assertions) { "dev" } else { "release" };
    println!("apm {} ({})", version, build);
}
```

**`apm/src/main.rs`** — three changes:
1. Add `Version` variant to the `Command` enum (no arguments needed).
2. Add `Version` to the help template command listing (Maintenance section).
3. Add dispatch arm: `Command::Version => cmd::version::run()`.

**Clap `--version` / `-V`:** Add `#[command(version)]` to the top-level `Cli` struct. Clap auto-generates `--version`/`-V` flags from `CARGO_PKG_VERSION`. This is separate from the `apm version` subcommand.

---

#### 2. Server — `/api/version` endpoint (`apm-server/src/`)

Add a `GET /api/version` route in `apm-server/src/main.rs` (wherever other routes are registered). No auth required.

```rust
async fn version_handler() -> impl IntoResponse {
    let build = if cfg!(debug_assertions) { "dev" } else { "release" };
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build": build,
    }))
}
```

---

#### 3. UI — version display (`apm-ui/src/`)

**`apm-ui/src/lib/api.ts`** (or equivalent): add `getVersion()` fetching `GET /api/version` → `{ version: string; build: string }`.

**`apm-ui/src/components/supervisor/SupervisorView.tsx`** (line 144 area):
1. Add `useQuery` for `getVersion()` with `staleTime: Infinity`.
2. Add `const [showVersion, setShowVersion] = useState(false)`.
3. Replace `<span>Supervisor</span>` with:
   ```tsx
   <span
     className="cursor-pointer select-none"
     onClick={() => setShowVersion(v => !v)}
     title="Click to toggle version"
   >
     Supervisor{showVersion && versionData ? ` · v${versionData.version} (${versionData.build})` : ''}
   </span>
   ```

No new Zustand store slice needed — `showVersion` is local component state.

---

**Order of changes** (each independently testable):
1. `apm/src/cmd/version.rs` + wire in `main.rs`
2. `apm-server` route + handler
3. `apm-ui` API helper + SupervisorView click toggle

### 1. CLI — `apm version` subcommand (`apm/src/`)

**`apm/src/cmd/version.rs`** — new file:
```rust
pub fn run() {
    let version = env!("CARGO_PKG_VERSION");
    let build = if cfg!(debug_assertions) { "dev" } else { "release" };
    println!("apm {} ({})", version, build);
}
```

**`apm/src/main.rs`** — three changes:
1. Add `Version` variant to the `Command` enum (no arguments needed).
2. Add `Version` to the help template's command listing (Maintenance section is a good fit, or a dedicated line).
3. Add dispatch arm in `main()`: `Command::Version => cmd::version::run()`.

**Clap `--version` / `-V`:** Add `#[command(version)]` to the top-level `Cli` struct. Clap will auto-generate `--version`/`-V` flags that print the version from `CARGO_PKG_VERSION`. This is independent of the `apm version` subcommand.

---

### 2. Server — `/api/version` endpoint (`apm-server/src/`)

**`apm-server/src/main.rs`** (or whichever file registers Axum routes):
- Add a `GET /api/version` route pointing to a handler `version_handler`.
- The handler returns JSON: `{"version": env!("CARGO_PKG_VERSION"), "build": "dev"|"release"}`.
- No auth required (public endpoint, same as health-check style routes).

```rust
async fn version_handler() -> impl IntoResponse {
    let build = if cfg!(debug_assertions) { "dev" } else { "release" };
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build": build,
    }))
}
```

---

### 3. UI — version display (`apm-ui/src/`)

**`apm-ui/src/lib/api.ts`** (or equivalent API module):
- Add `getVersion()` function: `GET /api/version` → `{ version: string; build: string }`.

**`apm-ui/src/components/supervisor/SupervisorView.tsx`** — two changes:
1. Add a `useQuery` call for `getVersion()` (with `staleTime: Infinity` — version won't change while server is running).
2. Add local `showVersion` boolean state (default `false`).
3. Replace the static `<span>Supervisor</span>` at line 144 with a clickable span:
   ```tsx
   <span
     className="cursor-pointer select-none"
     onClick={() => setShowVersion(v => !v)}
     title="Click to toggle version"
   >
     Supervisor{showVersion && versionData ? ` · v${versionData.version} (${versionData.build})` : ''}
   </span>
   ```

No new Zustand store slice is needed — `showVersion` is purely local UI state.

---

### Order of changes

1. `apm/src/cmd/version.rs` + wire into `main.rs` (CLI subcommand + `--version` flag)
2. `apm-server` route + handler (`/api/version`)
3. `apm-ui` API helper + SupervisorView click toggle

Each step is independently testable and has no dependency on the others.

### Open questions


### Amendment requests


### Code review


## History

| When | From | To | By |
|------|------|----|----|
| 2026-04-12T08:46Z | — | new | philippepascal |
| 2026-04-12T08:49Z | new | groomed | apm |
| 2026-04-12T08:50Z | groomed | in_design | philippepascal |
| 2026-04-12T08:54Z | in_design | specd | claude-0412-0850-ea48 |
| 2026-04-12T09:00Z | specd | ready | apm |
| 2026-04-12T09:00Z | ready | in_progress | philippepascal |

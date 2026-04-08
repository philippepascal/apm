+++
id = 71
title = "open_editor should split EDITOR/VISUAL on whitespace to support flags"
state = "closed"
priority = 5
effort = 1
risk = 1
author = "claude-0329-1430-main"
agent = "claude-0329-1430-main"
branch = "ticket/0071-open-editor-should-split-editor-visual-o"
created_at = "2026-03-29T23:40:38.875035Z"
updated_at = "2026-03-30T02:02:46.501095Z"
+++

## Spec

### Problem

`open_editor` in `apm/src/cmd/review.rs` (and `new.rs`) calls `Command::new(&editor).arg(path)` where `editor` is the raw value of `$VISUAL` or `$EDITOR`. `Command::new` treats the entire string as the binary name, so `EDITOR="zed --wait"` fails with "No such file or directory" because the OS looks for a binary literally named `"zed --wait"`.

The Unix convention is that `$EDITOR` may contain flags. `git`, `less`, and most tools that invoke `$EDITOR` split on whitespace: first token is the binary, the rest are prepended arguments.

### Acceptance criteria

- [x] `open_editor` splits `$VISUAL` / `$EDITOR` on whitespace; first token is the binary, remaining tokens are prepended as arguments before the file path
- [x] `EDITOR="zed --wait"` launches `zed` with args `["--wait", "<path>"]`
- [x] `EDITOR="vim"` (no flags) continues to work unchanged
- [x] Fix applies to `open_editor` in both `review.rs` and `new.rs`

### Out of scope

- Shell expansion (quotes, env vars) inside `$EDITOR` — simple whitespace split matches git and most Unix tools

### Approach

In both `open_editor` functions, replace `Command::new(&editor).arg(path)` with:

```rust
let mut parts = editor.split_whitespace();
let bin = parts.next().unwrap();
let mut cmd = Command::new(bin);
cmd.args(parts).arg(path);
```

## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-29T23:40Z | — | new | claude-0329-1430-main |
| 2026-03-29T23:40Z | new | in_design | claude-0329-1430-main |
| 2026-03-29T23:41Z | in_design | specd | claude-0329-1430-main |
| 2026-03-29T23:42Z | specd | ready | apm |
| 2026-03-29T23:43Z | ready | in_progress | claude-0329-1430-main |
| 2026-03-29T23:44Z | in_progress | implemented | claude-0329-1430-main |
| 2026-03-29T23:55Z | implemented | accepted | apm |
| 2026-03-30T02:02Z | accepted | closed | apm-sync |
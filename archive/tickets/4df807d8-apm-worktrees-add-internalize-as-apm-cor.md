+++
id = "4df807d8"
title = "apm worktrees --add: internalize as apm-core function, remove from public CLI"
state = "closed"
priority = 0
effort = 3
risk = 2
author = "claude-0330-0245-main"
agent = "claude-0330-0245-main"
branch = "ticket/4df807d8-apm-worktrees-add-internalize-as-apm-cor"
created_at = "2026-03-30T06:15:20.855321Z"
updated_at = "2026-03-30T18:08:26.456420Z"
+++

## Spec

### Problem

`apm worktrees --add <id>` is documented in `apm.agents.md` and used by agents
for spec-writing states (`new` → `in_design`, `ammend` → `in_design`). But per
`TICKET-LIFECYCLE.md`, worktree provisioning is always an internal step driven
by another command — it is never a user or agent action in its own right.

`apm start` already provisions worktrees internally and prints the path. The
spec-writing path lacks an equivalent: agents must call `apm worktrees --add`
manually, which leaks an implementation detail into the public CLI and agent
instructions.

The fix: move worktree provisioning into an `apm-core` function shared by all
commands that need it, have `apm state <id> in_design` auto-provision and print
the worktree path (mirroring `apm start`), and remove `--add` from the public
`apm worktrees` interface. Update `apm.agents.md` to remove the manual
`apm worktrees --add` calls.

### Acceptance criteria

- [x] `apm state <id> in_design` provisions a worktree for the ticket's branch if one does not yet exist
- [x] `apm state <id> in_design` reuses the existing worktree if one already exists for the branch
- [x] `apm state <id> in_design` prints the worktree path to stdout as the last line of output (after the state-change line)
- [x] `apm state <id> in_design` prints the worktree path for both the `new → in_design` and `ammend → in_design` transitions
- [x] `apm worktrees --add <id>` is no longer a recognised flag; passing it exits with a non-zero status and an error message
- [x] `apm worktrees` (list) and `apm worktrees --remove <id>` continue to work unchanged
- [x] `apm start <id>` continues to provision worktrees and print the path, with no behaviour change
- [x] `apm.agents.md` no longer references `apm worktrees --add`; the spec-writing workflow shows `apm state <id> in_design` as the single command that both transitions and provisions

### Out of scope

- Auto-provisioning for state transitions other than `→ in_design` (e.g. `→ in_progress` remains the domain of `apm start`)
- Merging the default branch into the ticket branch on `in_design` (that merge only happens in `apm start`)
- Removing `apm worktrees --remove` or the default list behaviour
- Changing where worktrees are stored (`config.worktrees.dir`)
- Adding any new `apm worktrees` subcommand to replace `--add`
- Changing the output format of `apm state` for transitions other than `→ in_design`

### Approach

**1. Extract shared provisioning into `apm-core/src/git.rs`**

Add a public function:

```rust
/// Find the worktree for `branch` or create one under `worktrees_base`.
/// Returns the canonical worktree path. Idempotent.
pub fn ensure_worktree(root: &Path, worktrees_base: &Path, branch: &str) -> Result<PathBuf> {
    if let Some(existing) = find_worktree_for_branch(root, branch) {
        return Ok(existing);
    }
    let wt_name = branch.replace('/', "-");
    std::fs::create_dir_all(worktrees_base)?;
    let wt_path = worktrees_base.join(&wt_name);
    add_worktree(root, &wt_path, branch)?;
    Ok(find_worktree_for_branch(root, branch).unwrap_or(wt_path))
}
```

Both `start.rs` and `state.rs` will call this instead of duplicating the find-or-create logic.

**2. Update `apm/src/cmd/start.rs`**

Replace the inline worktree provisioning block (the `wt_name` / `worktrees_base` / `find_worktree_for_branch` / `add_worktree` sequence) with a single call to `git::ensure_worktree(root, &worktrees_base, &branch)?`. No other behaviour changes.

**3. Update `apm/src/cmd/state.rs`**

After the existing commit step, add a block that fires only when `new_state == "in_design"`:

```rust
if new_state == "in_design" {
    let worktrees_base = root.join(&config.worktrees.dir);
    let wt = git::ensure_worktree(root, &worktrees_base, &branch)?;
    println!("{}", wt.display());
}
```

The existing `println!("{id}: {old_state} → {new_state}")` line runs before this block, so stdout for an `in_design` transition will be two lines:

```
4df807d8: new → in_design
/path/to/worktree
```

**4. Remove `--add` from `apm/src/cmd/worktrees.rs` and `apm/src/main.rs`**

- Delete the `add: Option<String>` field from the `Worktrees` CLI struct in `main.rs`
- Delete the `fn add(...)` function and its call-site in `worktrees.rs`
- Leave the list and `--remove` paths untouched

**5. Update `apm.agents.md`**

Replace the two spec-writing workflow blocks (for `new` and `ammend` states) that reference `wt=$(apm worktrees --add <id>)` with the updated pattern:

```bash
apm state <id> in_design   # provisions worktree; prints path as last line
# use the printed path with git -C:
git -C <printed-path> add tickets/<id>-<slug>.md
git -C <printed-path> commit -m "ticket(<id>): write spec"
```

Also remove or update the "Startup" section note that mentions `apm worktrees --add` as a provisioning mechanism, and update the `MAIN WORKTREE RULE` paragraph that currently lists both `apm worktrees --add <id>` and `apm start <id>` as provisioning commands — it should list only `apm state <id> in_design` and `apm start <id>`.

**Order of changes:** core first (step 1), then callers (steps 2–3), then CLI cleanup (step 4), then docs (step 5). Run `cargo test --workspace` after step 4 to confirm nothing is broken before touching docs.

### Open questions



### Amendment requests



### Code review



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-30T06:15Z | — | new | claude-0330-0245-main |
| 2026-03-30T06:20Z | new | in_design | claude-0330-0245-main |
| 2026-03-30T06:24Z | in_design | specd | claude-0330-0245-main |
| 2026-03-30T06:28Z | specd | ready | apm |
| 2026-03-30T06:30Z | ready | in_progress | claude-0330-0245-main |
| 2026-03-30T06:34Z | in_progress | implemented | claude-0329-1200-b7f2 |
| 2026-03-30T14:26Z | implemented | accepted | apm |
| 2026-03-30T18:08Z | accepted | closed | apm-sync |
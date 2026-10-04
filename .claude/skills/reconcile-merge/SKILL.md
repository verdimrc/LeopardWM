---
name: reconcile-merge
description: Finish merging the local main branch into the haha branch of LeopardWM after the user has run `git merge --no-commit --no-ff main` — verify preconditions, resolve conflicts per the agreed policy, fix build/clippy/tests, and update CHANGELOG-HAHA.md, leaving all commits to the user. Only run when the user invokes /reconcile-merge.
disable-model-invocation: true
---

# Reconcile the main → haha merge

The user owns every step that moves refs: switching to `haha`, refreshing `main`,
running `git merge --no-commit --no-ff main`, and every commit. Never run `git merge`, `git commit`,
`git add`, `git fetch`, `git pull`, `git checkout`/`git switch`, or anything that
updates `main` or changes branches. You only edit files. Your job starts after the user's `git merge --no-commit --no-ff main`: verify the
assumptions, then take the merge from wherever it stopped to done.

Never push. Never use destructive git commands (`reset --hard`, `checkout .`,
`clean -f`, `merge --abort`) without asking first.

## 1. Verify preconditions

All read-only. If any check fails, stop and tell the user what's wrong; do not
proceed or try to fix it yourself.

1. `git branch --show-current` must print `haha`.
2. No rebase in progress: neither `.git/rebase-merge` nor `.git/rebase-apply` may
   exist.
3. `main` must be fresh:
   - `git rev-parse main` must equal `git rev-parse origin/main`. If they differ,
     tell the user local `main` and `origin/main` disagree.
   - `git ls-remote origin refs/heads/main` must also match. This queries the
     remote without changing local refs, and catches a stale `origin/main` (the
     user hasn't fetched recently). If the command itself fails (offline, auth),
     tell the user and ask whether to proceed on the local refs alone.
4. Determine which state the user's merge left the repo in:
   - **A. Merge in progress** (conflicts, or the user used `--no-commit`):
     `git rev-parse -q --verify MERGE_HEAD` succeeds. `MERGE_HEAD` must equal
     `git rev-parse main` — otherwise the user merged something other than the
     current `main`; stop. Show the user `git status --short` so they can confirm
     every pending change comes from the merge.
   - **B. Merge already committed** (git auto-committed a conflict-free merge):
     no `MERGE_HEAD`, `HEAD` is a merge commit (`git rev-parse -q --verify HEAD^2`
     succeeds), `HEAD^2` equals `git rev-parse main`, and `git status --short` is
     empty.
   - **Neither:** if `git merge-base --is-ancestor main HEAD` fails, the user
     hasn't merged yet — tell them to run `git merge --no-commit --no-ff main` and stop. If it
     succeeds but `HEAD` isn't that merge, main was merged earlier; tell the user
     and stop.
5. Show the user the main tip hash and the incoming commits — `git log --oneline
   HEAD..main` in state A, `git log --oneline HEAD^1..HEAD^2` in state B — then
   continue.

In the steps below, "haha's side" means `HEAD` in state A and `HEAD^1` in state B.

## 2. Route by state

- State A with conflicts (`git diff --name-only --diff-filter=U` is non-empty):
  go to step 3.
- State A without conflicts: go to step 4.
- State B: go to step 4. The merge commit already exists, so build, Clippy, and
  test fixes will be separate fix-up commits on top of it (the user commits them).

## 3. Resolve conflicts

**Baseline stance:** main is upstream; haha is a feature layer on top. Adopt main's
structure, refactors, and fixes, then re-apply haha's intent on top. Where haha
deliberately diverges in behavior, haha's behavior wins.

**Understand both sides first**, for each conflicted file:
- `git diff $(git merge-base HEAD main) main -- <file>` (what main changed)
- `git diff $(git merge-base HEAD main) HEAD -- <file>` (what haha changed)
- `git log $(git merge-base HEAD main)..main -- <file>` and
  `git log $(git merge-base HEAD main)..HEAD -- <file>` (why each side changed it).

Commit messages are the ground truth for intent. In particular, haha's commit
messages tell you whether a haha change is a deliberate behavior divergence from
main (a `feat`, or a fix to main's behavior) versus incidental code that happens to
sit in the conflicted region.

**`CHANGELOG-HAHA.md` is a secondary check only.** It can be stale, and it
documents user-facing behavior, so internal fixes and small features may be
missing from it. Use it like this:

- *Before resolving:* skim its Features and Fixes for a quick map of where haha's
  deliberate divergences live (tray, placement, monitors, overview, ...). This only
  tells you where to look harder; it never decides a resolution.
- *When it disagrees with the commit messages:*

  | Commit messages say | Changelog says | Action |
  |---|---|---|
  | Deliberate divergence | Nothing | Trust the commit. Note it for the step 9 audit as a possible missing entry. |
  | Nothing (incidental change) | Lists a related behavior | Don't treat the hunk as a deliberate divergence on the changelog's word. The behavior may live elsewhere, or the entry is stale. Flag it to the user. |
  | Deliberate divergence | Matches | Keep haha's behavior with confidence. |

Keep a running list of every mismatch; it feeds the regression check in step 7
and the changelog audit in step 9.

**Rules by conflict type:**

| Conflict type | Resolution |
|---|---|
| Main refactored, renamed, or moved code that haha edited | Take main's version; re-port haha's change onto the new shape. |
| Both added independent things next to each other (fields, match arms, hotkeys, struct members, tests, imports) | Keep both. |
| Main fixed a bug that haha also fixed differently | Prefer main's fix; keep haha's only if it covers cases main's doesn't. Flag it to the user. |
| Haha deliberately diverges from main's behavior (per haha's commit messages; e.g. no `config.save()` from tray/hotkey toggles, new windows open on the monitor under the cursor, RTL layout, the one-shot placement toggle, tray badge) | Keep haha's behavior; take main's surrounding changes. |
| Both changed the same logic toward different outcomes | Stop and ask. Show both intents. |
| Both claim the same hotkey chord, config key, or IPC variant | Stop and ask. |
| `Cargo.lock` | Take main's; rebuild to regenerate. |
| `CHANGELOG.md`, version numbers, release files | Main's. |
| `CHANGELOG-HAHA.md`, `TODO-HAHA.txt` | Haha's. |
| Tests | Keep both. Adjust main's tests only where they fail because of haha behavior, after confirming that is the cause. |

Never silently delete either side's logic, and never pick a side when the two sides
mean different things.

**Approval before editing:** list every conflict hunk in a table (file, conflict
type, proposed resolution), and wait for the user's approval before editing. The
"keep both" and "main's refactor plus haha's re-port" rows may be presented as a
batch; the stop-and-ask rows must each get an explicit decision.

## 4–6. Fix build, Clippy, and tests

Run in order, fixing each before moving on:

1. `cargo build --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`

Merges compile cleanly as text but often break semantically, because main's new
code doesn't know about haha's additions. Breakage seen in past merges, as
examples of what to look for (not a checklist — they may not recur):
- Struct literals missing haha's fields (`WindowRule`, `CompiledWindowRule`,
  `SharedState`): use `..Default::default()` rather than listing every field.
- Exhaustive matches missing haha's `IpcCommand` variants.
- Clippy: `too_many_lines` (extract a helper), `too_many_arguments`,
  `collapsible_match`.
- Hotkey count and golden-set assertions (`test_hotkey_config_default` in
  `crates/daemon/src/config.rs`, the frozen expected set in
  `crates/ipc/src/hotkeys.rs`) need haha's extra bindings.
- Main's tests using `u64::MAX - 1` as a window ID collide with haha's
  `DESKTOP_PEEK_HWND`; switch them to plain IDs.
- Code that reads real Win32 state in a shared path (e.g. the foreground-window
  stale-focus guard) needs `#[cfg(not(test))]` so tests stay hermetic.
- Expected values that differ only because of haha behavior (e.g. outer-gap
  scroll offsets): update them after confirming haha is the cause.

If a fix requires choosing between main's and haha's behavior (not just adapting
code to compile or a test to haha's known behavior), stop and ask — same rule as
conflict resolution.

## 7. Validate

`pwsh -NoProfile -File tools/check.ps1`. The Clippy and Test lines must both print
`pass`. The Tools tests stage fails on this machine because Python is not
installed ("Python was not found"); report it, but it is not a blocker. Any other
Tools-stage failure is a real failure.

**Regression check against `CHANGELOG-HAHA.md`:** for each Features and Fixes entry,
spot-check that the behavior survived the merge — e.g. grep for its config key,
hotkey action id, or IPC variant. If a listed behavior's code disappeared or
changed shape, the merge may have silently dropped a haha feature: stop and flag
it. Add any stale entries you find to the mismatch list.

## 8. Hand off for commit

Do not stage or commit anything, and don't draft commit messages. The user stages
and commits; your hand-off must make clear which files need their attention.

First confirm no conflict markers remain in the files you resolved:
`git grep -n -E '^(<<<<<<<|=======|>>>>>>>)' -- <files>` must find nothing.

Then report the files in three groups:

| Group | Meaning | Git state |
|---|---|---|
| Conflicted, now resolved | Had conflicts; you edited them per the step 3 decisions | Unstaged — the user must `git add` them |
| Fix-ups | Edited in steps 4–7 to fix build, Clippy, or tests | Unstaged — the user must `git add` them |
| Merged cleanly | Brought in by the merge without conflicts; you did not touch them | Already staged by git |

In state A the user concludes their merge with `git add` + `git commit` and git's
default merge message; in state B there are no conflicted files, and the user
commits the fix-ups themselves.

Wait for the user to confirm they've committed before starting step 9, so the
changelog audit runs against the committed merge.

## 9. Update CHANGELOG-HAHA.md

1. Set the header base to the merged main tip:
   ```
   ## [Unreleased] — diff between `<main-tip>` .. HEAD
   ```
2. Re-audit every entry: each must be haha-only and backed by the diff. Check
   commits with `git merge-base --is-ancestor <commit> <main-tip>` (success means
   the commit is already in main).
3. Drop anything main now covers. Drop fixes to haha's own bugs. Add anything
   missing. Start from the mismatch list collected in steps 3 and 7: possible
   missing entries and stale entries.
4. Show the user the drafted changes before writing them.
5. After approval, write the file. Do not commit it.

## 10. Report

Preconditions checked, conflicts and how each was resolved, fix-ups made, check
results, changelog changes. No push.

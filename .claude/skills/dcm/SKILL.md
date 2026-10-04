---
name: dcm
description: Draft a commit message for the currently staged changes in LeopardWM, following .claude/rules/commit-messages.md. Never commits. Only run when the user invokes /dcm.
disable-model-invocation: true
---

# Draft commit message

Produce a commit message for the user to use. Never run `git add`, `git commit`, or anything
that changes the index or refs — drafting only.

The rules live in `.claude/rules/commit-messages.md` (already loaded in this session). That file
is the single source of truth; follow it exactly. This skill is the procedure around it.

## 1. Read the diff being committed

1. `git rev-parse -q --verify MERGE_HEAD` — if a merge is in progress, this commit concludes it:
   tell the user the merge commit uses git's default message and stop.
2. `git diff --cached --stat` and `git diff --cached`.
   - Nothing staged: use `git diff HEAD --stat` and `git diff HEAD`, and say the draft covers
     unstaged changes (the user still has to stage them).
3. `git diff --stat` — if there are also unstaged changes, say the draft covers only the staged
   part.
4. `git log --oneline -5` for the repo's `type(scope)` conventions.

Base the draft only on that diff — not on what was discussed in the session.

## 2. Understand the change before writing

- What does the user experience differently? (feature, fixed bug, behavior change)
- For a fix: what was the visible symptom, and is its cause obvious from the subject?
- Does the subject need a term a reader may not know?
- Is this one change, or two distinct ones?

If the user-visible effect isn't clear from the diff, say so and ask rather than guess.

## 3. Draft

Follow `.claude/rules/commit-messages.md`: `type(scope): subject`, a body only when needed, and
the session's attribution trailer.

## 4. Check before showing

Go through this list against the draft. Fix anything that fails, then show it.

- [ ] Subject describes what the user experiences, in plain language — not mechanics.
- [ ] Subject has no internal names (structs, functions, config keys, IPC variants) or internal
      concepts ("placeholder", "sentinel", "override").
- [ ] A bug fix reads "<thing> no longer <bad behavior>".
- [ ] The symptom is stated literally: no vague words ("drifts"), no unverified numbers.
- [ ] No double-meaning words (e.g. "permanent").
- [ ] Body omitted if the subject is self-explanatory; otherwise it only defines unfamiliar
      terms, gives "Previously, …" / "This fix …" for a non-obvious fix, or lists two distinct
      changes as bullets. It doesn't restate the diff.
- [ ] None of the words to avoid ("clean", "checks out", "landing").

## 5. Output

Show the message in a single code block, ready to paste, followed by one line saying what it
covers (staged changes, or staged-only if unstaged changes also exist). Nothing else.

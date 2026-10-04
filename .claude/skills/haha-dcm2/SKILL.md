---
name: haha-dcm2
description: Draft two commit messages for the currently staged changes in LeopardWM — always a one-liner and a version with a detail message. Never commits. Only run when the user invokes /haha-dcm2.
disable-model-invocation: true
---

# Draft commit message — always two versions

Same as `/haha-dcm`, except version B (with a detail message) is always produced, even when the
body is borderline. The user picks.

Read `.claude/skills/haha-dcm/SKILL.md` and follow it exactly — steps 1 (read the diff),
2 (understand the change), 4 (checks), and 5 (output) — with these differences:

- **Step 3:** always draft both versions with the same first line:
  - **Version A — one-liner:** `type(scope): subject` plus the trailer, no body.
  - **Version B — with a detail message:** the same first line plus the most useful body the
    rules allow: the steps that trigger the bug, an unguessable cause or fix
    ("Previously, …" / "This fix …"), an unfamiliar term, a new config key's example syntax
    (only that), or one bullet per part ("X changed:
    previously …; now …", or "(new)"; several changes to one part nested under it), dropping any bullet that only repeats the first line.
- **Step 4:** run every check on both versions, but don't drop version B for being guessable.
  Its body must still follow the rules: short, no restating the diff, no explaining what a
  documented setting does.
- **Step 5:** always show both blocks. Under version B, say in a few words what its body adds
  — or "adds little beyond the subject" if that's the honest assessment.

Never run `git add`, `git commit`, or anything that changes the index or refs.

---
name: haha-dcm
description: Draft one or two commit messages for the currently staged changes in LeopardWM — a one-liner, and when warranted, a version with a detail message. Never commits. Only run when the user invokes /haha-dcm.
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
2. `git diff --cached --stat` and `git diff --cached` — the staged changes are the only input.
   If nothing is staged, tell the user to stage what they want to commit, and stop.
3. `git diff --stat` — only to detect unstaged changes. Never draft from them; if any exist,
   say the draft covers only the staged part.
4. Look at the staged paths: if they mix unrelated changes (e.g. a code fix plus `.claude/`
   tooling), draft for the main change, name the other paths, and give the exact
   `git restore --staged <paths>` to split them off. Don't join unrelated changes with `;`.
5. `git log --oneline -5` for the repo's `type(scope)` conventions.

Base the draft only on that diff — not on what was discussed in the session.

## 2. Understand the change before writing

- What does the user experience differently? (feature, fixed bug, behavior change)
- For a fix: what was the visible symptom, and under what condition does it happen (e.g. only
  when the new window takes focus)? That condition belongs in the subject.
- Could a reader predict the cause and the fix from the subject alone? If so, no body.
- Does the subject need a term a reader may not know?
- Is this one change, or two distinct ones?

If the user-visible effect isn't clear from the diff, say so and ask rather than guess.

## 3. Draft

Follow `.claude/rules/commit-messages.md` and the session's attribution trailer. Produce one or
two versions with the same first line:

- **Version A — one-liner.** Always. `type(scope): subject` plus the trailer, no body.
- **Version B — with a detail message.** Only if a body is warranted: the reader couldn't
  predict it from the subject (an unguessable cause or fix, an unfamiliar term, the steps that
  trigger the bug, or the parts of one feature). If nothing qualifies, there is no version B.

## 4. Check before showing

Go through this list against each version. Fix anything that fails, then show them. The body
checks decide whether version B exists at all.

- [ ] Subject describes what the user experiences, in plain language — not mechanics.
- [ ] Subject has no internal names (structs, functions, config keys, IPC variants) or internal
      concepts ("placeholder", "sentinel", "override").
- [ ] The first line is at most 79 characters, measured with
      `printf '%s' '<first line>' | wc -m` — not estimated.
- [ ] A bug fix reads "<thing> no longer <bad behavior>".
- [ ] The condition under which the bug happens is in the subject.
- [ ] The symptom is stated literally: no vague words ("drifts"), no unverified numbers.
- [ ] No double-meaning words (e.g. "permanent").
- [ ] Noun phrases are compressed ("a newly focused window"), the subject reads naturally
      ("exiting desktop peek", not "desktop peek exit"), and the verb agrees with it.
- [ ] `;` joins only parts of one change set; unrelated changes are split off instead.
- [ ] Several parts: the first line only names them and says which changed and which are new
      ("update /haha-dcm; add /haha-dcm2"). No behavior described there.
- [ ] Each part is something the user uses (command, feature, behavior), not a file. A file
      that only supports a part (rules file, helper, test) is folded into that part — not named
      in the first line, no bullet of its own.
- [ ] Version B: could I have guessed this body from the subject? If yes, drop version B —
      regardless of diff size.
- [ ] Version B's body only defines unfamiliar terms, gives the steps that trigger the bug
      (naming a setting it depends on is fine), gives "Previously, …" / "This fix …" for a
      cause or approach a reader couldn't predict, or gives one bullet per part ("X changed:
      previously …; now …", or "(new)"; several changes to one part are nested under
      "X changed:"). It doesn't restate the diff or explain what a
      documented setting does.
- [ ] Each bullet on its own: does it add something the first line doesn't say? Drop any that
      only repeat it.
- [ ] None of the words to avoid ("clean", "checks out", "landing").

## 5. Output

Show each version in its own code block, ready to paste:

- **Version A (one-liner)**, then
- **Version B (with detail)**, only if it exists, with a few words on what the body adds that
  the subject doesn't.

Then one line saying what they cover (staged changes, or staged-only if unstaged changes also
exist) and the measured length of the first line. If unrelated changes are staged, add the
`git restore --staged <paths>` command to split them off. Nothing else.

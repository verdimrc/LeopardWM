# Commit messages

Loaded automatically in every Claude Code session in this repo. Check every draft against these
rules **before** showing it — don't draft and wait to be corrected.

## Source

Draft only from the diff being committed: `git diff --cached --stat` and `git diff --cached`
(or `git diff HEAD` if nothing is staged). Never from what was discussed in the session. If staged
and unstaged changes are mixed, say which one the draft covers.

## Format

`type(scope): subject`, then the attribution trailer given in the session.

## Subject (one-liner)

- Describe what the user experiences, in plain language. Not mechanics.
- No internal names: struct/function names, config keys, IPC variants, or internal concepts
  ("placeholder", "sentinel", "override", "fraction cache").
- Bug fix shape: "<thing> no longer <bad behavior>", e.g.
  `fix(daemon): moving a window between monitors no longer gains or loses pixels`.
- Be literal about the symptom. No vague words ("drifts its width") and no unverified numbers
  ("1-2 pixels off") — say exactly what happens ("gains or loses pixels").
- Avoid words with a second meaning in this project, e.g. "permanent" (reads as "saved to
  config.toml"). Use "default", "persistent setting", or rephrase.
- Two separate changes may share a subject joined by `;`, e.g.
  `feat(daemon): extend new window placement; tray status badge`.

## Body (detail)

Omit it when the subject is self-explanatory. When needed, keep it short:

- Define any term in the subject a reader may not know, e.g. "External tools are anything
  subscribed to LeopardWM's workspace updates over IPC, such as a status bar."
- For a fix whose cause isn't obvious, use two short paragraphs:
  "Previously, <what went wrong and why>." then "This fix <how, in plain terms>."
- For two distinct changes, use a short bullet list, one bullet per change.
- Never restate the diff line by line. A message longer than the diff is a red flag.

## Merges

When concluding a `/reconcile-merge`, the merge commit uses git's default message
("Merge branch 'main' into haha"); don't draft one.

## Words to avoid

- "clean" / "cleaner" — say what is or isn't included instead.
- "checks out" — say "accurate", or state the specific fact confirmed.
- "landing" (as in "is that landing?") — ask the literal question ("is that correct?").

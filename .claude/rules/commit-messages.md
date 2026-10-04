# Commit messages

Loaded automatically in every Claude Code session in this repo. Check every draft against these
rules **before** showing it — don't draft and wait to be corrected.

## Source

Draft only from the staged changes: `git diff --cached --stat` and `git diff --cached`. If nothing
is staged, say so and stop — don't fall back to unstaged changes. Never draft from what was
discussed in the session. If unstaged changes also exist, say the draft covers only the staged
part.

If the staged diff mixes unrelated changes (e.g. a code fix plus `.claude/` tooling), they belong
in separate commits: draft for the main change, name the other paths, and give the exact
`git restore --staged <paths>` to split them off.

## Format

`type(scope): subject`, then the attribution trailer given in the session.

There is no hard length limit on the first line. Make it as short as it can be while staying
plain and humane: every word should carry what the user gets, or how they turn it on.

## Subject (one-liner)

- Describe what the user experiences, in plain language. Not mechanics.
- No internal names: struct/function names, config keys, IPC variants, or internal concepts
  ("placeholder", "sentinel", "override", "fraction cache").
- Exception: a new feature the user turns on with a config key. The subject names the key and
  what it gives the user, e.g.
  `feat(daemon): layout.default_width_preset_monitor_overrides to set the per-monitor width preset`.
- Bug fix shape: "<thing> no longer <bad behavior>". The thing can be the action that goes
  wrong, e.g. `fix(daemon): exiting desktop peek no longer hides a newly focused window`.
- If the bug only happens under a condition, the condition is part of the subject:
  "…hides a newly focused window", not "…hides a new window". Leaving it out misstates the bug.
- Be literal about the symptom. No vague words ("drifts its width") and no unverified numbers
  ("1-2 pixels off") — say exactly what happens ("gains or loses pixels").
- Avoid words with a second meaning in this project, e.g. "permanent" (reads as "saved to
  config.toml"). Use "default", "persistent setting", or rephrase.
- To keep it short, compress noun phrases rather than dropping meaning: "a newly focused
  window", not "a window opened during desktop peek that takes focus". Prefer a natural action
  subject ("exiting desktop peek") over noun stacks ("desktop peek exit").
- Grammar: a gerund subject takes a singular verb ("exiting desktop peek no longer hides…").
- A commit with several parts: the first line just names the parts and says which changed and
  which are new, joined with `;` — e.g. `chore(claude): update /haha-dcm; add /haha-dcm2` — and
  stops there. Don't describe what they do in the first line; that goes in the detail.
- A part is something the user uses — a command, feature, or behavior — never a file. A file
  that only supports a part (a rules file, helper, test) goes under the part it serves: never
  name it in the first line or give it its own bullet.
- `;` joins only parts of one change set, e.g.
  `feat(daemon): extend new window placement; tray status badge`. Unrelated changes are separate
  commits (see Source).

## Body (detail)

Default to a one-liner. Add a body only if a reader couldn't predict it from the subject —
if the body's content is guessable from the subject, delete it. Diff size is irrelevant.

- Body needed: `moving a window between monitors no longer gains or loses pixels` — nobody can
  guess from that how a move changes a width (repeated rounding) or how it was fixed
  (remembering the proportion).
- No body: `rename /reconcile-merge to /haha-rcm and /dcm to /haha-dcm` — there is nothing to add.
- Borderline: `exiting desktop peek no longer hides a newly focused window` — the cause and fix
  are implied, but the trigger (opening a window from the revealed desktop while "focus new
  windows" is on) isn't. A trigger-steps body is worth offering as the alternative version.
- Never explain what a setting does — the config and docs already describe it. Naming a setting
  as part of the steps that trigger a bug is fine (see below).

When a body is needed, keep it short:

- Define any term in the subject a reader may not know, e.g. "External tools are anything
  subscribed to LeopardWM's workspace updates over IPC, such as a status bar."
- Give the steps that trigger a bug when a reader couldn't guess them, e.g. "Triggered by
  opening a window while desktop peek is on, e.g. double-clicking a file on the revealed
  desktop, with "focus new windows" enabled."
- For a new config key, show its syntax with an example value, e.g.
  `default_width_preset_monitor_overrides = { "2" = 3 }`. Nothing else: fallbacks, validation,
  and settings-window behavior are mechanics.
- For a fix whose cause or approach a reader couldn't predict, use two short paragraphs:
  "Previously, <what went wrong and why>." then "This fix <how, in plain terms>."
- For a commit with several parts, one bullet per part. A changed part reads
  "X changed: previously …; now …". A new part is marked "(new)". If one part has several
  changes, write "X changed:" and nest one "previously …; now …" bullet per change under it.
  Apply the predictability test to each bullet: drop any bullet that only repeats the first
  line.
- Never restate the diff line by line. A message longer than the diff is a red flag.

## Merges

When concluding a `/haha-rcm`, the merge commit uses git's default message
("Merge branch 'main' into haha"); don't draft one.

## Words to avoid

- "clean" / "cleaner" — say what is or isn't included instead.
- "checks out" — say "accurate", or state the specific fact confirmed.
- "landing" (as in "is that landing?") — ask the literal question ("is that correct?").

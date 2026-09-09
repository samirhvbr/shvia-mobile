# Changelog

Entries follow the commit-message format (`version - comment`), newest first — the same
convention as the sibling repositories. This file did not exist until 0.6.16; earlier
history lives in the git log.

## 0.6.24 - the release workflow was the one action left unpinned, and it made the ruler red

`toda_action_do_ci_esta_pinada_por_sha`, added in 0.6.19, has been **failing in master
since the day it was written**: `release.yml:44` carried `actions/checkout@v5`, a moving
tag. The ruler was authored in the same delivery as `release.yml`'s sibling workflow and
never run against the tree it landed in.

Pinned to `fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09 # v5`, the format `ci.yml` already
uses. This is the whole point of the ruler: a tag is a pointer its owner can move, and
`release.yml` runs with `contents: write`.

**Measured.** 7/7 green — the suite could not go green before this.

## 0.6.23 - the git hooks arrive from repodocs and are enabled here

Both hooks of the standard now run here: `commit-msg`, which checks the shape of
the subject (`X.Y.Z - description`), refuses a Conventional Commits prefix and a
vague message, **and checks that the subject's `X.Y.Z` is the version this commit
carries in `version.md`**; and `pre-push`, which compares the local `version.md`
against the remote default branch for a repeated version and for one that moves
backwards.

The hook does **not** check the language and could not: what it measures is the
shape and the number.

Until now the commit rule lived here only as prose in `CLAUDE.md`, and prose is
what gets forgotten at the end of a long session. On 07/09/2026 the hooks were
enabled in 3 clones out of 58, and two repositories of the fleet were measurably
off the norm with nothing to say so.

Escape hatch, declared in both: `REPODOCS_NO_HOOK=1`. It exists so the hooks stay installed —
a guard with no declared bypass gets bypassed with `--no-verify`, which switches
off every guard at once. In a fresh clone, enable them with
`git config core.hooksPath tools/git-hooks`.

## 0.6.20 - COMMIT-RULE replaces the COMMITTER delegation: the agent commits again

The `PS — Commits: a skill COMMITTER cuida disso` block in this repository's
agent instructions said that committing and pushing were not the agent's job,
because a cron cycle would package the commit from the changelog entry. That
skill was switched off across the fleet on 03/09/2026 — `.committer.yml` here is
`enabled: false` and nothing reads it any more. Switching off the automation did
not unwrite the delegation, so the instruction stayed behind pointing at a cycle
that no longer runs: an agent reading it stops at a dirty working tree that
nothing is watching.

Replaced by the `COMMIT-RULE` echo block, whose single source is
[samirhvbr/repodocs `docs/versioning.md`](https://github.com/samirhvbr/repodocs/blob/master/docs/versioning.md#who-commits-and-when)
(ADR-016 there). It says the opposite and says it in one place for the whole
fleet: the agent commits, nothing is reported as finished until it is committed,
one subject per commit, and a large delivery is split into blocks grouped by
subject. Being a delimited block rather than per-repo prose is the point — the
COMMITTER rollout put bespoke wording into 42 files and left no mechanical way
to find and replace it when the decision behind it was reversed.


## 0.6.19 - clippy was red where CI never looked, and four guards become tests

`mod tests` sat in the middle of `src-tauri/src/lib.rs` with `run()` declared below it.
`clippy::items_after_test_module` rejects any item declared after the test module, so
`cargo clippy -- -D warnings` was **already failing in this repository before
02/09/2026** and nobody saw it, because CI here did not run clippy. Same family as
finding **F-20**, which had measured only the DESKTOP. The test module moved to the end
of the file.

Four checks that were written prose became executable tests: the URL classifier (the
local shell and the server's three faces are internal, every other origin is external),
the `AGENTS.md` × `CLAUDE.md` mirror, every CI action pinned by SHA, and every document
reachable by link.

`scripts/sync-version.mjs` grew the propagation this needs; the iOS project files and
`tauri.conf.json` follow the same version.

## 0.6.19 - A ficha da App Store deixa de ser um arquivo que ninguém sabia que existia

Finding **D-DOC-10** of the September 2026 review.

> Entry body in English per `~/.claude/CLAUDE.md` (02/09/2026); the heading stays in
> Portuguese because COMMITTER derives the commit message from it.

- 🟢 **The README now lists all four documents**, `loja-ficha.md` included. That file is the
  App Store Connect listing, ready to paste, with Apple's character limits annotated — written
  on 04/08 and never linked from anywhere, in a repository whose submission is pending. The
  cost of an orphan document is not the file; it is somebody rewriting what was already done,
  which here would have meant redrafting store copy under submission pressure.

- 🟢 **New ruler `todo_doc_e_alcancavel`**, which the README itself now points at, so the rule
  and its guard are in the same place. This repo has no `docs/README.md` and does not need
  one — four documents fit in the README — but "fits in the README" only works while somebody
  keeps putting them there.

**Measured.** 6/6 green, clippy clean. Reversion: removing the document list from the README
→ red, naming `docs/loja-ficha.md`.

## 0.6.18 - O CI passa a rodar teste e clippy (que estava vermelho), e as actions ficam pinadas por SHA

Findings **F-09** and **F-21** of the September 2026 review, plus a red clippy found on the
way.

> Entry body in English per `~/.claude/CLAUDE.md` (02/09/2026); the heading stays in
> Portuguese because COMMITTER derives the commit message from it.

- 🟡 **`cargo clippy -- -D warnings` was red in this repository, and nobody could see it.**
  `clippy::items_after_test_module`: `mod tests` sat in the middle of `src-tauri/src/lib.rs`
  with `run()` below it. It was invisible because **the CI here ran only `cargo check`** —
  neither `cargo test` nor `cargo clippy`. Same family as finding F-20, which measured only
  SHVIA-DESKTOP; this repository was never measured. `mod tests` moved to the end of the
  file, with a comment saying why it must stay there.

- 🟢 **The CI now runs `cargo test` and `cargo clippy -- -D warnings`** (F-09). Without this
  the new rulers below would be decoration: a guard nobody executes is an instruction.

- 🟢 **Every `uses:` is pinned to a commit SHA**, with a comment naming the version (F-09).

- 🟢 **`AGENTS.md` and `CLAUDE.md` are byte-identical below the H1** (F-21). Both files
  already demanded this of themselves and both violated it — the diverging block was the
  "Leia também" pointer, written so that each file names the other, which **cannot** be
  identical in both. The rule was impossible to satisfy. The pointer is now symmetric and
  `agents_e_claude_sao_espelho` checks it.

**Two new rulers** in `src-tauri/src/lib.rs`: `agents_e_claude_sao_espelho` and
`toda_action_do_ci_esta_pinada_por_sha`.

**Measured.** 5/5 green (was 3 tests, and the suite was not run by CI), clippy clean — it
was red before. Reversions: one extra line in `AGENTS.md` → red; a new workflow with a loose
action → red.

## 0.6.17 - Os números que as lojas leem entram no sync, e o versionCode passa a se recusar a diminuir

Finding **F-24** of the September 2026 technical review.

> Entry body in English per `~/.claude/CLAUDE.md` (02/09/2026); the heading stays in
> Portuguese because the commit message is derived from it.

### The defect, measured with the repository at 0.6.16

| carrier | value | |
|---|---|---|
| `gen/apple/project.yml` | 0.6.8 | eight releases behind |
| `gen/apple/*/Info.plist` | 0.6.8 | idem |
| `gen/android/.../tauri.properties` | **0.2.2** | 🔴 a different numbering scheme altogether |

`sync-version.mjs` covered `package.json`, `tauri.conf.json`, `Cargo.*` and the locks — and
stopped there. `tauri ios build` regenerates part of `gen/`, but a clean clone and an Xcode
session opened by hand use what is in git: two sources of truth for the number the store
sees. The README already recorded that `gen/apple` "ficou defasado" once, missing
`NSFaceIDUsageDescription` — same family of defect.

### 🔴 The half that can actually break a release

Google Play refuses a build whose `versionCode` is not greater than the last published one.
Deriving it from semver is easy; deriving it *safely* is the point.

The scheme is `major*1_000_000 + minor*10_000 + patch*100` — room for 99 minors, 99 patches,
and a spare hundred per patch for re-submissions of the same version, which is exactly the
case that makes people hand-edit the file and lose sync in the first place.

And the script **refuses to write a lower code**, exiting 1 with the reason. Writing it would
produce a build the Play Console rejects, and submission is the most expensive moment to
discover that.

The old `versionCode` was 2002; the derived one is 61600 — it goes up, so nothing is
stranded.

### `--verificar`, and the loop it closes

0.6.16 added CI for this repository and its comment said, explicitly, that version checking
was left out **because this script had no check mode** — putting the step in before the
script existed would be a CI that pretends to measure, which is finding F-24 itself rather
than its fix.

The mode now exists, the step is in the workflow, and that comment was rewritten to describe
what the file does. A comment that survives the thing it describes is how doc-versus-code
drift starts.

Measured: a carrier out of step exits 1 and names the file; a `versionCode` that would
decrease exits 1 and says why.

## 0.6.16 - O repositório ganha CI, e a primeira coisa que ele mede são os advisories do Rust

Findings **G-24** (no CI anywhere) and **DEP-3 / F-31** (advisories never measured) of the
September 2026 technical review.

> Entry body in English per `~/.claude/CLAUDE.md` (02/09/2026); the heading stays in
> Portuguese because the commit message is derived from it.

### 🔴 The `time` pin had a vulnerability, exactly where F-31 said it would

`time = "=0.3.41"` — documented debt because of `wry`'s `cookie` — carried
**RUSTSEC-2026-0009**, a stack-exhaustion DoS. F-31 called this "the typical case where an
advisory would go unnoticed", and it was right.

The pin **moved with no code change**: `=0.3.47` resolves and `cargo check` passes. The
comment said "remove when wry/cookie accept the new `time`"; that had already happened and
nobody re-checked.

### The CI, and what it deliberately leaves out

`.github/workflows/ci.yml` runs `cargo deny check advisories` and `cargo check --locked` on
the host target. Building for Android/iOS needs SDK, keystore and provisioning — an
infrastructure decision on its own, and a CI that only runs the expensive thing does not run.

⚠️ **Version checking is absent on purpose**: this repository's `sync-version.mjs` has no
`--verificar` mode (the DESKTOP one does). Adding the step before the script exists would be
a CI that pretends to measure — which is finding F-24 itself, not its fix.

`src-tauri/deny.toml` carries sixteen documented exceptions for Tauri's GTK3 tree and one for
`quick-xml`, which ships in the binary and is pinned by `plist`. Each entry has a written
reason: an exception is debt with a reason and a way to re-check it.

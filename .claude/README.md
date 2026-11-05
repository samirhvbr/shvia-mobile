# Claude Code profile — ShvIA Mobile

This project's `.claude/` follows the Blue3/samirhvbr house pattern: permission
posture, and nothing that chooses a model. Stack: **Tauri 2 (Rust) + web shell
(npm/Vite) for iOS and Android**, with no sidecar — the server is remote, so
**no database and no secret runs here**. (The Python sidecar named in
`settings.json`'s comment and in `.gitignore` was copied from the desktop profile;
this repository has none.)

## Files

| File | Role |
|---------|-------|
| `settings.json` | The **active** profile (versioned). Today = `defaultMode: plan`, `effortLevel: xhigh`, a short security **deny-list** and an **ask-list**. It chooses **no model**. |
| `launch.json` | Launch configuration `shvia-mobile-ui`: `npm run dev` on port 1420 (the startup-page preview). |
| `skills/loop-work/`, `prompts/` | Vendored copy of the loop-work skill and its prompt templates; see `skills/loop-work/VERSION.md`. Change it upstream, not here. |

> The **allow-list** (shortcuts that avoid repeated prompts) is deliberately
> **not** in `settings.json`: granting the agent a permission is **your** act.
> Apply the block below by hand when you want fewer prompts.

## Recommended allow-list (paste into `permissions.allow`)

```jsonc
"allow": [
  "Read", "Edit", "Write",
  "Bash(git status:*)", "Bash(git diff:*)", "Bash(git log:*)",
  "Bash(git show:*)", "Bash(git branch:*)",
  "Bash(git add:*)", "Bash(git commit:*)", "Bash(git push:*)",
  "Bash(node -c:*)", "Bash(node --check:*)",
  "Bash(npm run dev:*)", "Bash(npm run build:*)", "Bash(npm run tauri:*)",
  "Bash(npm install:*)", "Bash(npx tauri info:*)",
  "Bash(cargo check:*)", "Bash(cargo build:*)", "Bash(cargo test:*)",
  "Bash(cargo fmt:*)", "Bash(cargo clippy:*)",
  "Bash(python -m py_compile:*)", "Bash(python3 -m py_compile:*)",
  "Bash(pytest:*)", "Bash(bats:*)", "Bash(shellcheck:*)"
]
```

## Rules worth remembering

- **The model is your choice, not this repository's** (repodocs ADR-027). You
  pick it per session with `/model`, and a subagent inherits the session's
  model. `settings.json` carries no `model` and no `fallbackModel`, and its
  `env` carries no `ANTHROPIC_MODEL`, `ANTHROPIC_DEFAULT_*_MODEL` or
  `CLAUDE_CODE_SUBAGENT_MODEL`. There are no stand-by profiles to copy over
  `settings.json` either — `/model` is how the model changes.
- **Effort `max` goes through the env** (`CLAUDE_CODE_EFFORT_LEVEL=max`). The
  JSON `effortLevel` field accepts only `low/medium/high/xhigh` — `max` there is
  ignored.
- **The context window comes with the model the session is on**, not from
  anything written here. This repository does not set
  `CLAUDE_CODE_DISABLE_1M_CONTEXT`. On the Max plan the large window is
  included — use it well clear of the limit.
- **`defaultMode: plan`** — the agent plans before acting. It keeps the habit of
  reviewing structural changes before touching code.

## Deny-list and ask-list (already in `settings.json`)

- **deny:** reading key files (`*.pem`, `*.key`, `*.p8`, `*.p12`, `*.pfx`).
- **ask:** `rm -rf` and piping `curl`/`wget` into `sh`/`bash`.

That is all this file holds. It does **not** deny reading `.env` files, and it
does not list `git push --force`, `git reset --hard` or `git clean -fd`; any
protection against those comes from your own Claude Code settings, not from this
repository. **Do not loosen** the lists without a documented reason.

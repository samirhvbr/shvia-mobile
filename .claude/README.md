# Claude Code profile — ShvIA Desktop

This project's `.claude/` follows the Blue3/samirhvbr house pattern: permission
posture, and nothing that chooses a model. Target stack: **Tauri 2 (Rust) + web
shell (npm/Vite) + Python sidecar (arrives in F2)** — the server is remote, so
**no database and no secret runs here**.

## Files

| File | Role |
|---------|-------|
| `settings.json` | The **active** profile (versioned). Today = `defaultMode: plan`, `effortLevel: xhigh`, and only the security **deny-list**. It chooses **no model**. |

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

## Deny-list (already in `settings.json`)

Blocks reading `.env`/keys (`*.pem`/`*.key`/`*.p8`/`*.p12`/`*.pfx`), `rm -rf`,
`git push --force/-f`, `git reset --hard`, `git clean -fd` and
`curl|sh`/`wget|sh`. **Do not loosen** it without a documented reason.

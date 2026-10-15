# ShvIA Mobile

ShvIA Mobile is the iOS and Android client of [ShvIA](https://ai.shvia.org): an AI
workspace for conversations, projects, knowledge and engineering work. A small
Tauri 2 shell opens the hosted ShvIA Web interface in WKWebView or Android System
WebView. The server owns accounts, data, models and permissions; this repository
owns startup and native mobile integrations.

The bundle/application ID is `cloud.blue3.shvia`. [version.md](version.md)
identifies the shell; the hosted Web has its own version and release cycle.

## What the app adds

- Local startup screen, reachability checks and reconnect controls.
- Optional biometric startup/resume gate and supported device-credential fallback.
- File export through the system save sheet.
- Camera/microphone declarations for Web features and WebView speech output.
- iOS APNs registration, foreground notifications and notification-tap routing.
- Navigation rules for ShvIA, external links and a narrow GitHub sign-in exception.

See the [feature matrix and contracts](docs/current-state.md) for platform limits.

## Current status

The checked-in iOS project targets **iPhone, iOS 15 or later**. Android targets
**API 24 or later**, with compile/target SDK 36. APNs is implemented on iOS;
Android FCM, Universal/App Links, a custom URL scheme and incoming shares are not
implemented here.

Earlier iPhone/TestFlight use and a release build after the iOS 27 scene correction
are dated repository records, not acceptance of each later build. Android has a
debug-build CI workflow; compilation does not establish device acceptance. Store
approval, currently installed versions, production APNs and Play distribution
were **not inspected in the documentation review of 15 October 2026**.

Remote Code belongs to the hosted Web/Workspace services. The phone does not run
local Claude Code, Codex or SHVIA CLI processes. See
[Integration](docs/integration.md) for local desktop projects versus persistent
remote environments and the current server-side bridge.

## Getting started

Update your checkout, read [AGENTS.md](AGENTS.md) and use a working branch.

```bash
npm ci --no-audit --no-fund
npm run build
npm run prova:bump
npm run prova:icones
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Host checks cover shared Rust and document rules. Native mobile code needs its
own toolchain/device checks; see [Development](docs/development.md).

## Documentation

The [documentation index](docs/README.md) separates the current handbook from
dated store records, ADRs and the unfinished [roadmap](.continue/escopo-mobile.md).

- [Current architecture and features](docs/current-state.md)
- [Development and validation](docs/development.md)
- [Web, Desktop and Workspace integration](docs/integration.md)
- [Build, signing and stores](docs/operations.md)
- [Device smoke test](docs/smoke-test.md)
- [Shield assets and icon checks](docs/brand.md)

Related repositories:
[Web](https://github.com/samirhvbr/shvia-web),
[Desktop](https://github.com/samirhvbr/shvia-desktop) and
[Workspace](https://github.com/samirhvbr/shvia-workspace).

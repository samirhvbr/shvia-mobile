# Development and validation

Run commands from the repository root. Read [AGENTS.md](../AGENTS.md), update the
checkout and create a branch before editing.

Enable the repository's git hooks once per clone:

```bash
git config core.hooksPath tools/git-hooks
```

[commit-msg](../tools/git-hooks/commit-msg) refuses a subject that is not
`X.Y.Z - description` or whose `X.Y.Z` differs from the staged `version.md`.
[pre-push](../tools/git-hooks/pre-push) refuses a push whose `version.md` equals
or is lower than the version on the remote default branch. Neither runs a test
suite; both are generated from repodocs.

## Clean checkout and host checks

Use Node.js 22, npm and stable Rust to match [CI](../.github/workflows/ci.yml);
`prova:trava` runs TypeScript with `--experimental-strip-types`, so 22.6 is the
oldest Node that works.
Native host compilation needs Tauri's platform dependencies; Linux CI installs
WebKitGTK 4.1, GTK 3, Ayatana AppIndicator and librsvg development packages.

```bash
npm ci --no-audit --no-fund
npm run build
npm run prova:bump
npm run prova:icones
npm run prova:trava
npm run prova:push
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --locked --manifest-path src-tauri/Cargo.toml -- -D warnings
```

The build type-checks/bundles the startup page; `prebuild` also synchronizes version
carriers, so review its diff. `prova:bump` checks without writing; on a clean
checkout it reports Android's untracked `tauri.properties` as skipped, which is
expected. The icon check verifies the Shield seal; do not re-seal unexpected
differences to make it pass. `prova:trava` runs the biometric gate's decision table and
then the real `biometricGate`, `runLock` and `authOpts` against a fake native plugin;
`prova:push` evaluates the push-token delivery function (`src-tauri/src/push_token.js`,
the file Rust embeds) against stand-in page locations. Neither drives a device.

CI also runs `cargo deny check advisories` against
[deny.toml](../src-tauri/deny.toml), whose ignores each carry a reason. With
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny) installed, the local
equivalent is:

```bash
cargo deny --manifest-path src-tauri/Cargo.toml check advisories
```

Only `check advisories` is part of CI; 0.7.10 recorded that `check licenses`
fails on this repository's own unlicensed crates.

Rust tests cover navigation, return paths/picker suppression, names/base64,
the `saveFile` shim's argument names, the iOS push-token script's wrapper and the
filter on its callback, exclusive iOS export staging and overlapping
requests using temporary files, capabilities, iOS project invariants, the Android
manifest's permissions and features, Android resource XML comments, SHA-pinned
workflow actions, the agent-document mirror and doc reachability. Several of these
are source checks: they read files rather than run the app.
Host tests do not execute Face ID, Swift APNs, Android document providers or a
hosted conversation. Android CI separately builds one arm64 debug APK and reads
its permissions; it uploads no artifact and covers no signed release/device.
No checked-in workflow builds iOS.

## CI workflows

| Workflow | Runs on | Does |
|---|---|---|
| [CI](../.github/workflows/ci.yml) | Push to `master`/`main`, every pull request, manual | `npm ci`, advisories, `cargo test`, clippy, `prova:bump`, `prova:icones`, `prova:trava`, `prova:push` |
| [Android](../.github/workflows/android.yml) | Pull requests and `master` pushes that touch `src-tauri/`, `src/`, `plugins/`, npm manifests or the workflow; manual | arm64 debug APK, then permissions/features read from the APK |
| [Release](../.github/workflows/release.yml) | `master`/`main` pushes that change `version.md`; manual, optionally `backfill` | `tools/release.sh`: tag and GitHub Release for the version |

The jobs name `runs-on: ubuntu-latest` directly. The fleet's self-hosted-runner
block in [AGENTS.md](../AGENTS.md) describes jobs that read `vars.CI_RUNNER`; these
workflows do not, so setting that variable changes nothing here until a workflow
reads it. Every change bumps `version.md` and its carriers under `src-tauri/`,
so even a documentation change runs all three workflows once it merges.

## Startup-page preview

`npm run dev` serves `http://localhost:1420`. Use `/?hold` for connecting or
`/?hold=offline` for offline without automatic navigation. A browser has no native
biometric plugin; this validates only the bundled UI.

## Android

The [Gradle app](../src-tauri/gen/android/app/build.gradle.kts) uses minimum API 24,
compile/target SDK 36. CI pins JDK 17, NDK `28.2.13676358`, build-tools `35.0.0`.
Install these SDK packages and configure the machine's paths:

```bash
export JAVA_HOME="/path/to/jdk-17"
export ANDROID_HOME="/path/to/android-sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
rustup target add aarch64-linux-android
npm run tauri android dev
```

CI-equivalent debug build:

```bash
npm run tauri android build -- --debug --target aarch64 --apk
```

Expected APK: `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`.
Use `adb devices` and `adb install -r <apk>` for a connected phone, then run
the [smoke test](smoke-test.md). Other architectures need their Rust targets.

## iOS

Use macOS, full Xcode selected through `xcode-select`, the iOS SDK, CocoaPods,
XcodeGen and the appropriate Rust targets. Command Line Tools alone lack the
iOS SDK. The checked-in project targets iOS 15/iPhone.

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
npm run tauri ios dev
```

Intel simulators would need `x86_64-apple-ios`, but the checked-in
`project.yml` sets `ARCHS: [arm64]`, so that route is not verified. Physical
phones need developer mode and signing. See [Operations](operations.md) before
producing a release candidate.

`gen/apple` and `gen/android` are already versioned. Do not run `tauri ios init`
or `tauri android init` for routine setup: regeneration may replace customizations.
If needed, review the entire generated diff against Operations' invariants,
synchronize versions and check icons afterward.

Record commit, shell/Web versions, device/OS and command for each validation.
A green host suite does not close native acceptance, and a signed IPA does not
close installation, APNs delivery or store review.

# Development and validation

Run commands from the repository root. Read [AGENTS.md](../AGENTS.md), update the
checkout and create a branch before editing.

## Clean checkout and host checks

Use Node.js 22, npm and stable Rust to match [CI](../.github/workflows/ci.yml).
Native host compilation needs Tauri's platform dependencies; Linux CI installs
WebKitGTK 4.1, GTK 3, Ayatana AppIndicator and librsvg development packages.

```bash
npm ci --no-audit --no-fund
npm run build
npm run prova:bump
npm run prova:icones
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --locked --manifest-path src-tauri/Cargo.toml -- -D warnings
```

The build type-checks/bundles the startup page; `prebuild` also synchronizes version
carriers, so review its diff. `prova:bump` checks without writing. The icon check
verifies the Shield seal; do not re-seal unexpected differences to make it pass.

Rust tests cover navigation, return paths/picker suppression, names/base64,
exclusive iOS export staging and overlapping requests using temporary files,
capabilities, iOS project invariants, the agent-document mirror and doc reachability.
Host tests do not execute Face ID, Swift APNs, Android document providers or a
hosted conversation. Android CI separately builds one arm64 debug APK and reads
its permissions; it uploads no artifact and covers no signed release/device.
No checked-in workflow builds iOS.

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

Intel simulators need `x86_64-apple-ios`; physical phones need developer mode and
signing. See [Operations](operations.md) before producing a release candidate.

`gen/apple` and `gen/android` are already versioned. Do not run `tauri ios init`
or `tauri android init` for routine setup: regeneration may replace customizations.
If needed, review the entire generated diff against Operations' invariants,
synchronize versions and check icons afterward.

Record commit, shell/Web versions, device/OS and command for each validation.
A green host suite does not close native acceptance, and a signed IPA does not
close installation, APNs delivery or store review.

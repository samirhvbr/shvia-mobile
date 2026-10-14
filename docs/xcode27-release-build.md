# Release build on Xcode 27 — the swift-rs workaround, until it is fixed upstream

> **Update 14/10/2026 — on Tauri 2.12.1 (`tao` 0.37.1) the workaround below was NOT needed.**
> The first release store build of 0.7.7, after `cargo update -p tauri --precise 2.12.1`, ended in
> `BUILD SUCCEEDED` and a 3.9 MB IPA with only steps 0 and 1 done: no `cargo clean`, no
> `llvm-objcopy` on `libTauri.a`, no fingerprint removal. That was measured; **why** it works was
> not investigated (the new `tauri` build has its own `libTauri.a`, and `swift-rs` is still 1.0.8).
> Steps 2 to 5 are for the 2.11.x lock that built 0.7.6; keep this page until a machine on a 2.11.x
> lock no longer exists, then delete it.

On Xcode 27 (Swift 6.4) the **debug** iOS build works and the **release** iOS build fails at
the link. This page is the procedure that produced the 0.7.6 store build on 07/10/2026, and
the measurements behind each step. **Delete it when `swift-rs` publishes the fix** (see the
last section) and the plain command works again.

```bash
npm run tauri ios build -- --export-method app-store-connect
```

## Why it breaks

Xcode 27's SwiftPM *internalizes* the `@_cdecl` exports of a static Swift library (`nm` shows
them as local `t`), so the Rust side cannot link them. `swift-rs` 1.0.8 promotes them back to
global with `llvm-objcopy`, in two ways that matter here:

1. It needs rustup's **`llvm-tools`** component. Without it the only sign is a Cargo warning
   (`swift-rs: llvm-objcopy not found`) and a link error listing `_init_plugin_*`, `_log_stdout`,
   `_register_plugin` … as undefined.
2. It promotes **only the package's own object member** (`Tauri.o`, `tauri-plugin-dialog.o`, …).
   The runtime shim every archive embeds, `SwiftRs.o`, is skipped, so `_retain_object`,
   `_release_object`, `_string_from_bytes` and `_data_from_bytes` stay local. This is
   [swift-rs#81](https://github.com/Brendonovich/swift-rs/issues/81); the proposed fix,
   [#82](https://github.com/Brendonovich/swift-rs/pull/82), was open and unreleased on 07/10/2026.

## The procedure

**0. The machine.** `xcode-select -p` must be `/Applications/Xcode.app/Contents/Developer`
(`sudo xcode-select -s …`). The Tauri CLI starts `xcodebuild` without the caller's environment,
so `DEVELOPER_DIR` is ignored.

**1. Install the component, once per machine.**

```bash
rustup component add llvm-tools
```

**2. Make the Swift build scripts run again.** Cargo caches their output, so a component
installed afterwards changes nothing until they re-run. From `src-tauri/`:

```bash
cargo clean --release --target aarch64-apple-ios \
  -p tauri-plugin-dialog -p tauri-plugin-biometric -p tauri-plugin-opener \
  -p tauri-plugin-shvia-push -p swift-rs
```

**3. Run the release build.** It fails once more, now with only `_retain_object`,
`_release_object` and `_string_from_bytes` undefined. Everything the build script has to
produce now exists.

**4. Promote the shim's symbols in one archive.** One is enough: the other archives keep their
own local copies, and promoting in all of them is what duplicates globals.

```bash
OBJCOPY="$(rustc --print sysroot)/lib/rustlib/aarch64-apple-darwin/bin/llvm-objcopy"
ARCHIVE="$(find src-tauri/target/aarch64-apple-ios/release/build -name libTauri.a | head -1)"
"$OBJCOPY" --globalize-symbol=_retain_object --globalize-symbol=_release_object \
  --globalize-symbol=_string_from_bytes --globalize-symbol=_data_from_bytes "$ARCHIVE"
nm "$ARCHIVE" | grep -E ' T _(retain_object|release_object|string_from_bytes)$'   # three lines
```

**5. Make the `tauri` crate pick the patched archive up.** `rustc` *bundles* a static library
into the `.rlib` of the crate that links it, so the copy inside `libtauri-<hash>.rlib` is the
old one even though `libTauri.a` is now right. Remove only the library unit's fingerprint
(the directory that holds a `lib-tauri` file, **not** the `run-build-script…` one), by its
literal path:

```bash
ls src-tauri/target/aarch64-apple-ios/release/.fingerprint/ | grep '^tauri-'   # find the one with lib-tauri
rm -rf src-tauri/target/aarch64-apple-ios/release/.fingerprint/tauri-<hash-with-lib-tauri>
```

**6. Run the build command again.** Measured on 07/10/2026: `BUILD SUCCEEDED`, a 3.8 MB IPA.

## What was measured on the result

0 `.a`/`libapp` in the payload, 1 `PrivacyInfo`, `UIDeviceFamily = [1]`, `MinimumOSVersion` 15.0,
`DTSDKName iphoneos27.0`, version 0.7.6, the microphone, camera and Face ID strings and
`ITSAppUsesNonExemptEncryption = false` in `Info.plist`. Signed `Apple Distribution: BLUE3
TECNOLOGIA LTDA (S65UBCTPN5)` with `iOS Team Store Provisioning Profile: cloud.blue3.shvia`
(valid to 2027-07-17); entitlements `aps-environment = production`, `get-task-allow = false`.
The distribution certificate is cloud-managed: `security find-identity` shows none in the
keychain, and none is needed.

## When this page can go

When a `swift-rs` release includes the fix for #81, bump it (`cargo update -p swift-rs`), run
the release build **without** steps 2, 4 and 5, check it ends in `BUILD SUCCEEDED`, and delete
this file. Step 1 stays: the warning is a warning, not an error, so a machine without
`llvm-tools` fails only at the link ([#83](https://github.com/Brendonovich/swift-rs/issues/83)
asks for it to fail earlier).

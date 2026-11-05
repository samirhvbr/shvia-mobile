# Current architecture and features

Source review: 15 October 2026, rechecked against `master` at 0.7.14 on 5 November
2026. This describes implementation, not a store account or a build currently
installed on a phone. Changes proposed in open pull requests are not described here
until they merge.

## Runtime and ownership

[lib.rs](../src-tauri/src/lib.rs) creates one `main` WebView and opens the bundled
[startup page](../src/main.ts). It displays the Shield screen, checks reachability
and navigates to `https://ai.shvia.org`. The hosted Web owns chat, projects, files,
knowledge, model selection, account settings and responsive layout.

Login uses the WebView's same-origin session cookie. Local biometrics do not
replace server authentication or encrypt the account. No application database
or model runtime runs here. Device data still includes WebView cookies/storage/cache,
the local biometric preference, saved files, iOS export staging and APNs state.
Server/provider keys and store signing material are not shipped in the app.

## Platform matrix

| Capability | iOS | Android | Boundary |
|---|---|---|---|
| Hosted UI | WKWebView | System WebView | Hosted service, account and server configuration |
| Minimum target | iOS 15, iPhone family | API 24; compile/target SDK 36 | Source settings, not acceptance on all devices |
| Startup/reconnect | Implemented | Implemented | No offline conversation storage or AI execution |
| Biometric startup/resume gate | Implemented | Implemented | Requires plugin/device/lifecycle testing |
| Save a file | Export sheet after Documents staging | Document provider `content://` | User chooses destination; command takes no path |
| Camera/microphone | Usage strings declared | Permissions declared; hardware optional | Capture/transcription belongs to Web and needs device testing |
| Read aloud | WebView `speechSynthesis` | WebView `speechSynthesis` | Installed voices/engine vary |
| Push | APNs plugin implemented | No FCM implementation | Permission, signing and server delivery are separate |
| GitHub sign-in | Narrow in-app exception | Same rule | SSO that leaves GitHub opens externally |
| Local CLI/folder execution | Absent | Absent | Remote Code belongs to Web/Workspace |
| Custom URL scheme, Universal/App Links, incoming shares | Absent | Absent | Notification taps use a different route mechanism |

The host binary is for development/tests; it is not ShvIA Desktop and supplies
none of Desktop's local Code engines, menus or window features.

Other platform declarations in the native projects:

- iOS supports portrait and both landscape orientations (no upside-down) and
  requires `arm64` and `metal` devices ([project.yml](../src-tauri/gen/apple/project.yml)).
- The Android manifest keeps the Tauri template's Android TV entry: a
  `LEANBACK_LAUNCHER` category and the `android.software.leanback` feature marked
  not required. No TV layout or TV test exists; treat TV support as untested.
- Android debug builds allow cleartext traffic (`usesCleartextTraffic`); release
  builds do not. The ShvIA hosts are HTTPS-only either way.

## Connectivity

Startup fetches `/up` with `no-cors`, no cache and a six-second timeout. An opaque
response proves reachability, **not HTTP 200/application health**. Failure shows
reconnect controls; another automatic attempt starts five seconds after the
previous failure completes. The `online` event also retries. "Abrir mesmo assim"
bypasses this probe and still passes through the biometric gate. The splash footer
shows `v<shell version>` (from `package.json`, kept equal to `version.md`).

The remote offline banner reacts to an `offline` event (or a page that loads while
the WebView reports offline) with a same-origin `/up` probe and five-second timeout.
Unlike the startup probe, a non-2xx answer counts as a failure. While a failure is
shown, it probes every 15 seconds; `online` hides the banner, and tapping it reloads
the page. This is recovery UI, not continuous health monitoring. Reloading does not
guarantee preservation of unsent Web state.

## Biometric gate and lifecycle

The bundled origin stores `shvia_biometric` as `on`/`off`. On first available
biometric detection it offers opt-in; enabling requires authentication. Cold
start with the preference on asks to unlock. After a failed prompt, disabling
also requires authentication. Plugin options allow device-credential fallback
where supported.

The local page reports the active gate through `trava_biometrica`. On mobile
`WindowEvent::Resumed`, Rust sends a ShvIA page to the local gate, carrying only
path and query. Unlock reloads the Web page; it does not restore arbitrary DOM
state. The iOS scene manifest is part of this lifecycle contract: in scene mode,
tao 0.37.1 emits `Resumed` from `sceneWillEnterForeground` (Android: `onResume`).
That mapping is read from tao's source; it still needs device evidence on iOS 27.

A file-input click marks the remote page; one resume within ten minutes consumes
the mark and skips the gate to preserve picker results. This is a heuristic,
not proof that the resume came from the picker.

Limits: declining the initial offer is remembered and no general settings screen
here re-enables it. Since 0.7.12, with the lock on, biometrics that are unavailable
(a Face ID lockout, biometrics removed) or a failed status query no longer let the
app through: the gate asks for the device passcode (`allowDeviceCredential`). Until
then both cases proceeded without a gate, so someone holding an unlocked phone could
fail Face ID into a lockout and open ShvIA (external review, 26/10/2026). The one
exception is a device with no passcode at all: no lock is possible there, and the
app lets the person in, by the owner's decision of 27/10/2026. The decision is
`src/trava.ts`, checked by `npm run prova:trava`. Device acceptance on iOS and
Android is still required.

## Native commands, navigation and downloads

[Capabilities](../src-tauri/capabilities/servidor-shvia.json) grant remote ShvIA
pages only `salvar_arquivo`. The bundled origin
([default](../src-tauri/capabilities/default.json) and
[mobile](../src-tauri/capabilities/mobile.json) capabilities) gets `core:default`,
`opener:default`, `trava_biometrica` and, on iOS/Android, `biometric:default`.
No page receives direct filesystem/dialog permissions or a native
terminal/local-project bridge. `build.rs` registers both app commands with the
ACL, and a host test fails if a remote capability grants anything else.

Internal HTTPS hosts are exactly `ai.shvia.org`, `ia.shvia.org` and
`ia.blue3.com.br`. Ordinary external links, including `shvia.org`, open in the
system browser. GitHub HTTPS `/login`, `/login/*`, `/session` and `/sessions/*`
stay inside for OAuth. GitHub receives no remote capability and is excluded
from normal page-load injection. The asynchronous push path is separate, with its own URL and
host checks; see [Integration](integration.md) and its acceptance limits.

Each completed ShvIA page load receives one injected script with:

- `window.__shviaShellVersion`, the shell version. Web appends it to its top-bar
  version tag as `· app X.Y.Z` (`shviaVersionTag` in shvia-web `public/js/app.js`,
  checked at shvia-web 2.114.133), so the topbar shows both versions;
- the "Sistema Offline" banner described under [Connectivity](#connectivity);
- the file-picker mark used by the resume gate;
- the `saveFile` shim below and, once known, the iOS push token.

The bundled page's CSP limits its connections to itself, the three ShvIA hosts
and Tauri IPC. Adding a server host means changing `SERVER_HOSTS` in
[lib.rs](../src-tauri/src/lib.rs), the remote capability URLs and that CSP in
[tauri.conf.json](../src-tauri/tauri.conf.json) together
(ADR-003 in [Decisions](decisoes.md)).

The injected `window.__shviaCode` has only `saveFile({name,dataBase64})`:
`salvar_arquivo` keeps only the last path segment of the suggested name, drops
control and reserved characters (`< > : " | ? *`) and leading dots, cuts it to 120
characters and falls back to `arquivo`. It applies a nominal 50 MiB file-size guard
computed from the base64 length. Cancel returns `{saved:false,path:null}`; success
reports the suggested name rather than an Android provider URI. This is not
Desktop's `spawn`/`send` bridge.

iOS reserves that name in app Documents without replacing an existing file. A
collision returns an error before opening the export sheet, preserving the existing
document. Only one export may hold the native sheet at a time. Its staged file is
removed after success or cancellation, and on a write failure. A file left by an
interrupted process is treated as an existing file, not silently overwritten.
Android writes decoded bytes to the chosen provider URI. Test both paths,
cancellation and repeated names; host staging tests do not prove device acceptance.

## iOS notifications

The [Swift plugin](../plugins/tauri-plugin-shvia-push/ios/Sources/ShviaPushPlugin.swift)
adds APNs callbacks to the runtime AppDelegate, obtains a device token, presents
foreground notifications (banner, sound and badge) and passes tap routes to Rust.
The route is the notification payload's `route` key. It retains token/error state
and a pending cold-start route. Existing runtime APNs selectors produce an error
rather than being silently replaced. Rust only logs registration errors to
standard error; neither the page nor the user sees them.

Permission is requested once per process at the first completed ShvIA page load,
which can be the login page; this is not an authenticated-account check. Rust
keeps the hexadecimal token in memory and delivers `__shviaPushToken`,
`__shviaPushPlatform='ios'` and `shvia:push-token`. Web registers it using cookie/CSRF.
Permission, token registration and production delivery need separate evidence.

Tap routes must start with one slash and pass a character filter. Rust applies
`location.assign` in the current WebView; this is not a custom URL scheme.
Cold start, a locked/logged-out app and OAuth require device acceptance before
claiming they always return to the intended ShvIA screen.

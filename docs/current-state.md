# Current architecture and features

Source review: 15 October 2026. This describes implementation, not a store account
or a build currently installed on a phone.

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

## Connectivity

Startup fetches `/up` with `no-cors`, no cache and a six-second timeout. An opaque
response proves reachability, **not HTTP 200/application health**. Failure shows
reconnect controls; another automatic attempt starts five seconds after the
previous failure completes. The `online` event also retries. "Abrir mesmo assim"
bypasses this probe and still passes through the biometric gate.

The remote offline banner reacts to an `offline` event with a same-origin `/up`
probe and five-second timeout. While a failure is shown, it probes every 15 seconds;
`online` hides the banner. This is recovery UI, not continuous health monitoring.
Reloading does not guarantee preservation of unsent Web state.

## Biometric gate and lifecycle

The bundled origin stores `shvia_biometric` as `on`/`off`. On first available
biometric detection it offers opt-in; enabling requires authentication. Cold
start with the preference on asks to unlock. After a failed prompt, disabling
also requires authentication. Plugin options allow device-credential fallback
where supported.

The local page reports the active gate through `trava_biometrica`. On mobile
`WindowEvent::Resumed`, Rust sends a ShvIA page to the local gate, carrying only
path and query. Unlock reloads the Web page; it does not restore arbitrary DOM
state. The iOS scene manifest is part of this lifecycle contract.

A file-input click marks the remote page; one resume within ten minutes consumes
the mark and skips the gate to preserve picker results. This is a heuristic,
not proof that the resume came from the picker.

Limits: declining the initial offer is remembered and no general settings screen
here re-enables it. Unavailable biometrics or a failed status query proceeds
without an active gate. Test these cases before requiring this gate as an
organizational security control.

## Native commands, navigation and downloads

[Capabilities](../src-tauri/capabilities/servidor-shvia.json) grant remote ShvIA
pages only `salvar_arquivo`. The bundled origin can use biometrics and
`trava_biometrica`. No page receives direct filesystem/dialog permissions or a
native terminal/local-project bridge.

Internal HTTPS hosts are exactly `ai.shvia.org`, `ia.shvia.org` and
`ia.blue3.com.br`. Ordinary external links, including `shvia.org`, open in the
system browser. GitHub HTTPS `/login`, `/login/*`, `/session` and `/sessions/*`
stay inside for OAuth. GitHub receives no remote capability and is excluded
from normal page-load injection. The asynchronous push path is separate; see
[Integration](integration.md) and its acceptance limits.

The injected `window.__shviaCode` has only `saveFile({name,dataBase64})`:
`salvar_arquivo` sanitizes the suggested name and applies a nominal 50 MiB
file-size guard computed from the base64 length. Cancel returns `{saved:false,path:null}`; success reports
the suggested name rather than an Android provider URI. This is not Desktop's
`spawn`/`send` bridge.

iOS stages under that name in app Documents, opens the export sheet and attempts
to remove staging afterward. Android writes decoded bytes to the chosen provider
URI. Test both paths, cancellation and repeated names.

## iOS notifications

The [Swift plugin](../plugins/tauri-plugin-shvia-push/ios/Sources/ShviaPushPlugin.swift)
adds APNs callbacks to the runtime AppDelegate, obtains a device token, presents
foreground notifications and passes tap routes to Rust. It retains token/error
state and a pending cold-start route. Existing runtime APNs selectors produce
an error rather than being silently replaced.

Permission is requested once per process at the first completed ShvIA page load,
which can be the login page; this is not an authenticated-account check. Rust
keeps the hexadecimal token in memory and delivers `__shviaPushToken`,
`__shviaPushPlatform='ios'` and `shvia:push-token`. Web registers it using cookie/CSRF.
Permission, token registration and production delivery need separate evidence.

Tap routes must start with one slash and pass a character filter. Rust applies
`location.assign` in the current WebView; this is not a custom URL scheme.
Cold start, a locked/logged-out app and OAuth require device acceptance before
claiming they always return to the intended ShvIA screen.

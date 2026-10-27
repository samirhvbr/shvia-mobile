# Web, Desktop and Workspace integration

The mobile binary and hosted Web release independently. A server feature can
appear without a mobile update; a native capability needs a new binary. Verify
the deployed combination before promising it on a phone.

## Ownership

| Repository | Responsibility |
|---|---|
| [shvia-web](https://github.com/samirhvbr/shvia-web) | UI, accounts/authorization, chat/projects, model gateway, APIs and remote Code screen |
| This repository | Startup, mobile biometric gate, navigation, native export and iOS APNs |
| [shvia-desktop](https://github.com/samirhvbr/shvia-desktop) | Desktop shell, local CLI engines/folders/terminals and native Code bridge |
| [shvia-workspace](https://github.com/samirhvbr/shvia-workspace) | Persistent remote environments, repositories, engine sessions and remote filesystem/processes |

The phone's WebView is not a container or a copy of a desktop working directory.
Server-side projects/sessions can outlive a phone or tab, subject to the services'
session/authorization contracts. Resuming them across devices differs from
copying the phone's local UI state.

## Remote Code

The current Web composes a Workspace bridge with Desktop's bridge when available.
In a browser/mobile app it can use the Workspace half for a remote project; a local
desktop folder still needs Desktop. The mobile `saveFile` shim is deliberately
insufficient for local Code mode.

This repository has no native `spawn`, `send`, terminal, local folder picker,
Git write API or CLI credential writer. Remote engine/model selection, leases,
event polling, files, diffs and durable history belong to Web/Workspace. Remote
catalogues do not establish local engine availability. Reading a file is not
proof that an engine ran or committed work.

Read Web's current
[Remote Code screen](https://github.com/samirhvbr/shvia-web/blob/master/docs/FRONTEND/REMOTE-CODE.md)
and [API contract](https://github.com/samirhvbr/shvia-web/blob/master/docs/API/REMOTE-CODE.md).
The older read-only Workspace-tab guide does not describe the newer remote
project bridge. This review compared local Web source; it did not check its
deployment or execute a Workspace session on an installed phone.

Acceptance should open an existing project/session on a second device, preserve
project identity, read an existing file and continue a turn without starting an
unintended local session. See the [smoke test](smoke-test.md).

## Sessions, OAuth and notifications

Web login uses the normal same-origin cookie and CSRF flow. The app's cookie jar
is separate from the system browser. Biometrics gate local entry; they are not
an API key, organization policy or substitute for logout/revocation.

GitHub sign-in/consent paths stay inside to return to the same cookie jar. Ordinary
GitHub pages open externally. SSO through another identity provider leaves the
allowed path set; there is no mobile deep-link callback to recover automatically.
Test the organization's actual login path.

Swift sends token/error/route events to Rust. Tokens live in memory, are delivered
to the active WebView on receipt and reinjected on later ShvIA page loads. Web
listens for `shvia:push-token` and calls `POST /push/token` with cookie/CSRF. Its
logout flow attempts `DELETE /push/token`; network failure/server cleanup is a
Web operational concern.

Normal page-load injection is restricted to ShvIA hosts. Since 0.7.13 the
asynchronous token callback goes through the same filter: it evaluates the token
only when the webview is on a ShvIA page, and otherwise leaves it stored for the
next ShvIA load. Before that it evaluated into whatever page was current, GitHub's
sign-in included (external review, 26/10/2026). Tap routes still act on the current
page. Host tests check the filter in source; device acceptance during OAuth, logout
and the biometric gate is still required.

The [server APNs contract](https://github.com/samirhvbr/shvia-web/blob/master/docs/PUSH/PUSH-APNS-20260716.md)
owns provider credentials. The phone needs its signed push entitlement, user
permission and a network path. Android FCM has no implementation here.

## Downloads, voice and layout

Web reads authorized artifact bytes and calls
`window.__shviaCode.saveFile({name,dataBase64})`; native code validates the name
and encoded size and opens the system destination picker. Server artifact
authorization/quota is separate from native filesystem permissions.

Microphone capture, server transcription, attachments and speech output combine
Web UI with WebView support. Native permission declarations enable a request;
they do not prove working recording, service health, camera or voices. Keyboard,
project layout and touch controls largely belong to Web. Report both shell and
Web versions so failures reach the right repository.

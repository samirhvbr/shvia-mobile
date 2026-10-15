# Device smoke test

Run on the exact candidate build. A simulator covers some layout/biometric cases;
physical devices are needed for camera, microphone, cookie persistence, APNs and
real lifecycle evidence. See [Development](development.md) for installation and
[Operations](operations.md) for candidates. This checklist claims no new acceptance.

## Evidence record

```text
Date/time and tester:
Commit / shell version / build number:
Artifact checksum / installation route:
Device / OS / WebView:
Hosted Web / Workspace version when relevant:
Network (Wi-Fi, cellular, VPN):
Account role (no passwords, API keys or device tokens):
Per check: PASS / FAIL / NOT RUN, observed result, evidence and issue link.
```

## Common flows

| Check | Expected observation |
|---|---|
| Cold start | Shield splash/version, then hosted Web without a native error page |
| Offline startup | Retry and "Abrir mesmo assim" respond; restored network triggers retry |
| Network loss in Web | Offline probe/recovery UI; reconnect/reload without duplicate banners |
| Login/restart/logout | Intended cookie/account persists; logout removes access |
| Streaming | Response arrives progressively; test cellular and Wi-Fi |
| Projects/history | Correct existing project/conversation, usable long/touch content |
| Read aloud | Installed language/voice works or its absence is recorded |
| Microphone | Permit, record and transcribe; denial leaves other features usable |
| Attachments/camera | Select file/photo and upload; cancel/deny remains usable |
| Export | Save/open text, PDF/image and compare contents; cancel reports no success; repeat name |
| Keyboard/rotation | Accessible composer, safe areas, scrolling and landscape |
| External link | System browser, including landing and GitHub repositories |
| GitHub connection | Password/2FA round trip returns to the same account; test SSO separately |

Android needs real permission prompts and document-provider export. CI's merged
manifest checks do not execute these flows.

## Biometrics and lifecycle

1. In a fresh test state with biometrics enrolled, enable the offer and authenticate.
   In another fresh state, decline and verify the remembered choice. Do not clear a
   user's real app data for testing.
2. With the lock enabled, cold-start and background/foreground; the gate asks again.
   Cancel/fail/retry; disabling after failure requires authentication.
3. Test supported device-credential fallback and unavailable/failed status queries.
   The current implementation can proceed without a gate in those latter cases.
4. Return from a file picker with a selection and after cancellation; the one-resume
   exception should preserve the result. Then background normally and verify relock.
5. Verify the intended path returns after unlock; record draft/state loss because
   the gate reloads the page rather than guaranteeing in-memory restoration.

The iOS 27 scene correction needs testing on that OS; a successful archive does
not prove the lifecycle event reaches the gate.

## iOS APNs

On the signed candidate, record permission grant/denial, server registration
without copying the device token, and controlled delivery in foreground,
background and terminated states. Test taps when logged in, logged out, locked
and during GitHub OAuth. Match the signed entitlement to the server environment.
A token alone does not prove production delivery. Android FCM is unimplemented.

## Remote Code continuity

When enabled by the deployed Web/Workspace pair, open an existing remote project,
identify its repository/session and read an existing file. Send a turn, verify
engine/model provenance, events and changes; move to a second device and resume
the same project/session with consistent files/history. Record takeover/lease
prompts. The phone must not start a local CLI or treat its export shim as Desktop.
See [Integration](integration.md).

Keep evidence dated by build/platform. Historical [roadmap](../.continue/escopo-mobile.md)
and [publication](testflight-checklist.md) checkboxes do not close this candidate.

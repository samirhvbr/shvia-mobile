# Build, signing and store operations

Source-based procedure, 15 October 2026. It does not report live App Store Connect,
Google Play, signing accounts, APNs or installed builds. Earlier measurements
remain in [publication history](testflight-checklist.md) and
[App Review Notes](app-review-notes.md).

## Versions and delivery

[version.md](../version.md) is the shell version source. Policy requires a versioned
changelog entry/commit, including documentation changes. After bumping, run
`npm run version:sync` and `npm run prova:bump`. The script synchronizes npm,
Tauri, Cargo and checked-in iOS fields. Android's generated `tauri.properties`
is also synchronized if present; a clean checkout does not track it.

Android's script derives `versionCode` as
`major*1_000_000 + minor*10_000 + patch*100` and refuses to lower an existing code.
This scheme reserves two decimal places per component; larger components need a
deliberate scheme change. It compares the local generated file, not Play's
highest uploaded build.

iOS version/build fields follow the repository version. Each upload needs an
acceptable unused build number: inspect the exported binary and latest portal
upload, since export settings can manage numbering independently.

The [Release workflow](../.github/workflows/release.yml) publishes a GitHub tag/
Release when a version change reaches `master`/`main`. A feature PR does not
trigger it. A GitHub Release is separate from an IPA/APK build, store upload,
approval, deployment and installed acceptance.

## Generated project invariants

The native projects are committed. Regeneration needs an entire diff review:

| iOS invariant | Inspect |
|---|---|
| Bundle `cloud.blue3.shvia`, team `S65UBCTPN5`, iOS 15 | Tauri config, `gen/apple/project.yml`, generated project/plist |
| iPhone family `1` | Xcode settings and exported `UIDeviceFamily` |
| Microphone/camera/Face ID strings | `Info.ios.plist`, `project.yml`, generated and exported Info.plist |
| Scene manifest and export-compliance flag | Source templates, generated files and actual bundle |
| APNs environment | Template entitlement block, generated entitlement and actual signature |
| Privacy manifest included | Xcode resources and actual IPA contents |
| Rust archives linked, not copied as resources | `Externals` excludes, `embed:false`, IPA contents |
| Shield icons | Icon seal and actual exported icon |

XcodeGen's template regenerates plist/entitlements. Editing only an output can
be undone; that happened here with usage strings and APNs. Preserve both source
templates and generated invariants.

Android must retain `INTERNET`, `RECORD_AUDIO`, `CAMERA`, optional microphone/camera
features and its FileProvider. Inspect the **merged APK manifest**, not only
source. Preserve customizations when regenerating either platform.

## iOS candidate

Use [Development](development.md). A configured team is not proof this machine
can sign: verify selected Xcode, account/provisioning and distribution method
without copying credentials into Git.

```bash
npm ci --no-audit --no-fund
npm run version:sync
npm run prova:bump
npm run prova:icones
npm run tauri ios build -- --export-method app-store-connect
```

The [Xcode 27 investigation](xcode27-release-build.md) records that the Tauri
2.12.1/tao 0.37.1 store build did **not** need the old 2.11.x archive patch. Start
with a normal build; keep the workaround tied to its measured lock/linker error.

Inspect the actual IPA's version/build, iPhone family, minimum OS, usage strings,
scene/privacy manifests, icons, signature/team and APNs environment. No Rust `.a`
or `libapp` may be copied into the payload as a resource. Development/debug and
store APNs environments differ; match the actual signature to the server.

Upload through Organizer/Transporter or the team's supported authenticated flow.
Record checksum, processing result and selected build, then run the
[smoke test](smoke-test.md) on the candidate that will be submitted. An older
cable-installed build is different evidence.

## Android candidate

[Android CI](../.github/workflows/android.yml) builds an arm64 debug APK and checks
permissions. It supplies no release-signing/Play pipeline. The checked-in app
Gradle file has no release signing identity, so a local release build does not
establish a signed publishable AAB or keystore custody.

Before Play delivery, arrange the approved identity, release format, monotonic
code, current target requirements and artifact inspection. Keep keys/passwords
outside Git. Test a physical Android phone with the intended server version.
No Play publication or Android device acceptance occurred in this review.

## Submission and production checks

Recheck the current Web before reusing store text: sign-up, deletion, microphone,
privacy/support URLs, native features and review-account instructions must match
what a reviewer sees. Use a disposable account for deletion, not the review
account. Passwords belong in protected portal fields.

The [August listing](loja-ficha.md) has outdated account-provisioning claims;
[October Notes](app-review-notes.md) record a later sign-up check but remain dated.
Verify live store rules/requirements/status at submission time.

Device acceptance includes denial, background/foreground, picker return, export,
logout, notification cold start and OAuth. Remote Code additionally needs the
deployed Web/Workspace pair and second-device continuity. Report remaining failures
separately; neither source inspection nor a green host suite proves these flows.

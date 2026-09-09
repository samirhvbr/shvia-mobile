# App Review Information ▸ Notes — the text as submitted, and its review

The literal text that sits in the **Notes** field of App Store Connect, kept here
because it was living only in a browser form. Reviewed on 09/09/2026 against the
code, hours before the resubmission of build 0.6.5.

> **Why this file exists.** `docs/testflight-checklist.md` §3 carries a *proposed*
> Notes text that was never the one in the field — the field held a longer,
> different text nobody had copied back. Two versions of the same paragraph, one
> in a form and one in a repo, is how a claim drifts away from the product without
> anybody deciding it should.

---

## The text in the field on 09/09/2026 (3,387 characters, excerpt as read)

```text
ShvIA is a corporate AI workspace (B2B). Accounts are provisioned by each
organization's administrator — there is no public sign-up in the app; the
demo account above is pre-provisioned for review.

Native features beyond the web experience: Face ID app lock (opt-in card on
first launch), push notifications (APNs; a test notification can be triggered
for the demo account on request), camera/microphone for attachments and
voice dictation, native offline handling, and external links opening in
Safari. The in-app content is served from our own domain (ai.shvia.org)
only — general web browsing is not possible.
```

---

## Review — three problems, ordered by what they cost

### 🔴 1. Account deletion is not mentioned at all

This is **the guideline the app was rejected under** on 12/08/2026 —
5.1.1(v). The reviewer who reopens the submission reads this field first, and
today it does not say where the deletion lives. Everything else on this page is
optional next to this.

### 🔴 2. "voice dictation" contradicts the fix, and the Resolution Center answer

**Measured on 09/09/2026:** the guard is at `public/js/app.js:7648-7651`
(`iOSWebView` / `webSpeechUsavel`) in SHVIA-WEB, and the file served by
production has the **same md5 as the local master**
(`ebb16b0e8898dd8f91670f3e5ab71772`) — so the microphone control **is hidden**
inside the WebView. That was the fix for the *other* rejected guideline, 2.1(a).

The Notes promise the reviewer a feature the app deliberately no longer offers,
while the Resolution Center reply says the opposite in the same submission:
*"speech input is not offered inside the app's WebView, so the microphone control
is hidden."* A reviewer who goes looking for dictation does not find it — on the
exact guideline that already failed once.

The camera and microphone permissions stay in the bundle for **attachments**,
which is true and worth saying. Only "voice dictation" comes out.

### 🟡 3. The push promise is a commitment with an unverified last mile

*"a test notification can be triggered for the demo account on request"* is an
offer volunteered to someone who can hold us to it inside a review window.

**The client half is real** — this was checked, because the checklist said
otherwise and the checklist was wrong (see below): `tauri-plugin-shvia-push` is
wired in `src-tauri/src/lib.rs`, it injects `window.__shviaPushToken` and fires
`shvia:push-token`, and SHVIA-WEB's `app.js` reads that symbol in three places.
Build 0.6.5 is after 0.6.0, so the binary under review has it.

**What is NOT verified from here** is the last mile, and any one of these breaks
the promise silently:

- the **Push Notifications capability** on App ID `cloud.blue3.shvia` in the
  developer portal;
- `APNS_*` in the production `.env`, with `APNS_PRODUCTION=true`;
- `aps-environment` = **production** in the uploaded IPA — `project.yml` holds
  `development`, and the export to App Store Connect is what promotes it. The
  pre-upload audit for exactly this is still an open checkbox in §0.

**Either confirm the chain end to end before submitting, or drop the
parenthetical.** Claiming push exists is fine; offering an on-demand
demonstration is a different sentence.

### 🟡 4. Face ID is the strongest claim and the least exercised

True since 0.4.0, and it is the best answer to guideline 4.2 in the whole
paragraph. But **no build on a physical device has ever run that path** —
smoke-test item 10 is still open, and the card is the *first screen a reviewer
sees*. The note is not wrong; it is an invitation to tap the one control nobody
has tapped on real hardware.

### 🟢 What is right and should not be touched

- *"there is no public sign-up"* — answers the "how do we even get in" question
  before it is asked.
- *"served from our own domain (ai.shvia.org) only — general web browsing is not
  possible"* — this is the anti-4.2 argument, and it is well put.

---

## Proposed replacement

Same structure, one claim removed, one paragraph added. Fits the 4,000-character
field.

```text
ShvIA is a corporate AI workspace (B2B). Accounts are provisioned by each
organization's administrator — there is no public sign-up in the app; the
demo account above is pre-provisioned for review.

Account deletion (Guideline 5.1.1(v)): Settings -> System -> "Conta & zona de
risco" (Account & danger zone) -> "Quero excluir minha conta" (Delete my
account) -> confirm with the account password -> "Excluir definitivamente"
(Delete permanently). This permanently deletes the account and its content
(conversations, files, projects, memories and provider keys). A screen
recording of this exact flow on a physical iPhone is attached. The recording
uses a throwaway account (appletest@shvia.org), not the demo account above,
because the flow deletes the account for real; at the end of the video that
same address is typed back into the sign-in form and is rejected, which shows
this is a deletion and not a deactivation.

Native features beyond the web experience: Face ID app lock (opt-in card on
first launch), push notifications (APNs), camera and microphone for
attachments, native offline handling, and external links opening in Safari.
Speech input is not offered inside the app's WebView, so the microphone
control is hidden there; the usage descriptions remain in the bundle for
attachments. The in-app content is served from our own domain (ai.shvia.org)
only — general web browsing is not possible.
```

---

## Correction this review forced elsewhere

**`docs/testflight-checklist.md` §2.1 said the push client did not exist.** It
carried an unchecked *"(a) shell Tauri (ESTE repo)"* and a dated note reading
*"Reconferido em 30/07 (SHVIA-WEB 2.88.8): `grep __shviaPushToken public/js/app.js`
não acha nada"*. Both were true on 30/07 and false since **0.6.0** (04/08),
which shipped *"Push (APNs): cliente completo na casca"*. The measurement was
right; it was never re-run after the thing it measured changed. Corrected in the
same commit as this file.

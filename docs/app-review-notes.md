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

## The text for the 0.7.6 submission (07/10/2026)

Written after re-reading the Notes against the product, which is the check that failed
three times (`docs/rejeicao-20260910.md`). What was measured, and where:

| Claim | Measured | How |
|---|---|---|
| Public sign-up exists | `GET https://ai.shvia.org/register` → 200; sign-in page links "Create one here" | `curl`, 07/10/2026; the owner confirmed it is open **on purpose** |
| Phone optional, no date of birth, 18+ checkbox | `phone_ddi/ddd/number` optional; `first_name`, `last_name`, `email`, `password`, `terms`, `age_confirmation` required; no birth-date field | the served form, field by field |
| Deletion path, English | Settings → System → "Account and danger zone" → card "Delete my account" → button "I want to delete my account" → password → "Delete permanently" | `lang/en/settings.php` + `dashboard.blade.php` (shvia-web master 2.111.108) |
| Deletion path, Portuguese | Sistema → "Conta e zona de risco" → "Excluir minha conta" → "Quero excluir minha conta" → "Excluir definitivamente" | `lang/pt_BR/settings.php` |

The previous text wrote the pane as "Conta & zona de risco"; the `&` lives in a comment in the
Blade file, the string the screen shows is "Conta e zona de risco".

**Not measured from here, and to be checked before Submit:** the screens behind the login (this
session has no credentials), that `apple@shvia.org` is alive and its password matches the one in
App Store Connect, that the deletion video is attached to this version, and the microphone and
Face ID claims on the build being submitted (the smoke test on the iPhone covers them).

```text
ShvIA is an AI workspace delivered as a thin native client for our web application
(ai.shvia.org). Sign in with the demo account provided above. The in-app content is
served from our own domain only; general web browsing is not possible.

Sign-up (public): the sign-in screen has a "Create one here" link to a sign-up form.
It asks for first name, last name, email, password, acceptance of the terms and a
confirmation that the user is 18 or older. The phone number is optional and we do
not collect a date of birth.

Account deletion (Guideline 5.1.1(v)): Settings -> System -> "Account and danger
zone" -> "I want to delete my account" -> enter the account password -> "Delete
permanently". (Portuguese interface: Sistema -> "Conta e zona de risco" -> "Quero
excluir minha conta" -> "Excluir definitivamente".) This permanently deletes the
account and its content (conversations, files, projects, memories and provider keys);
it is not a deactivation. A screen recording of this exact flow on a physical iPhone
is attached. It uses a throwaway account (appletest@shvia.org), not the demo account
above, because the flow deletes the account for real.

Native features beyond the web experience: Face ID app lock (opt-in card on first
launch; the app locks again each time it returns from the background), push
notifications (APNs), camera for photos attached to a message, microphone for voice
dictation (the microphone button in the message composer records, and the audio is
transcribed on our server), saving a file to the phone through the system "save as"
sheet, and external links opening in Safari.
```

⚠️ The counter under the Notes field counts **down** (what is left of 4,000), as
recorded below. This text is about 1,600 characters (line breaks included), so it should read about `2,400`.

---

## The text in the field on 09/09/2026 (613 characters — the whole of it)

> 🔴 **The counter under the Notes field counts DOWN, and reading it as "used"
> produced a wrong finding here.** The field showed `3,387`; that was taken to
> mean the text was 3,387 characters long and that only 18% of it had been read,
> which turned into an alarm to stop and fetch the rest. It is the opposite: the
> text is **613** characters and 3,387 was the room left of 4,000.
>
> **Settled by writing, not by re-reading.** Pasting a block of `aaaa…` moved the
> counter from `3,389` **down** to `2,659` — adding characters lowered the number.
> Five seconds, and no amount of staring at the screen would have decided it,
> because the number carries no unit. The replacement text below later measured
> 1,408 characters and the field stopped at `2,592`, which closes the arithmetic.
>
> The alarm cost one round trip and was the safe direction to fail in — the unsafe
> one was pasting over 2,774 characters nobody had read. But the cheap test
> existed the whole time and was not run.

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

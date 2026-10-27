#!/usr/bin/env node
// The biometric gate's decision (src/trava.ts), case by case. Until 0.7.12 a failed status query, or
// biometrics reported unavailable (on iOS that includes a Face ID lockout), let the app through with
// the lock turned on (external review, 26/10/2026). Node strips the types of the .ts file it imports
// (Node 22.18+; the flag keeps older 22.x working).
import assert from "node:assert/strict";
import { decidirTrava, semCredencialNoAparelho } from "../src/trava.ts";

const casos = [
  // [description, reading, expected]
  ["lock off: go on", { emTauri: true, pref: "off", statusFalhou: false, status: { isAvailable: true } }, "seguir"],
  ["first run with biometrics: offer", { emTauri: true, pref: null, statusFalhou: false, status: { isAvailable: true } }, "oferecer"],
  ["lock on with biometrics: ask", { emTauri: true, pref: "on", statusFalhou: false, status: { isAvailable: true } }, "travar"],
  ["lock on, Face ID locked out: ask (passcode)", { emTauri: true, pref: "on", statusFalhou: false, status: { isAvailable: false, errorCode: "biometryLockout" } }, "travar"],
  ["lock on, biometrics removed: ask (passcode)", { emTauri: true, pref: "on", statusFalhou: false, status: { isAvailable: false, errorCode: "biometryNotEnrolled" } }, "travar"],
  ["lock on, status query failed in the shell: ask", { emTauri: true, pref: "on", statusFalhou: true }, "travar"],
  ["lock on, device without a passcode: go on (owner, 27/10/2026)", { emTauri: true, pref: "on", statusFalhou: false, status: { isAvailable: false, errorCode: "passcodeNotSet" } }, "seguir"],
  ["first run without biometrics: no offer", { emTauri: true, pref: null, statusFalhou: false, status: { isAvailable: false, errorCode: "biometryNotEnrolled" } }, "seguir"],
  ["dev server in a browser: go on", { emTauri: false, pref: "on", statusFalhou: true }, "seguir"],
];
for (const [nome, leitura, esperado] of casos) {
  assert.equal(decidirTrava(leitura), esperado, nome);
}
assert.equal(semCredencialNoAparelho({ code: "passcodeNotSet", message: "x" }), true, "iOS no passcode");
assert.equal(semCredencialNoAparelho({ code: "noDeviceCredential" }), true, "Android no screen lock");
assert.equal(semCredencialNoAparelho("passcodeNotSet"), true, "error as a plain string");
assert.equal(semCredencialNoAparelho({ code: "userCancel" }), false, "a cancel is not a missing passcode");
assert.equal(semCredencialNoAparelho({ code: "authenticationFailed" }), false, "a failed attempt keeps the lock");
console.log(`✓ trava biométrica: ${casos.length + 5} casos`);

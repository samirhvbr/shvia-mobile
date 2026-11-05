#!/usr/bin/env node
// The biometric gate's decision (src/trava.ts), case by case. Until 0.7.12 a failed status query, or
// biometrics reported unavailable (on iOS that includes a Face ID lockout), let the app through with
// the lock turned on (external review, 26/10/2026). Node strips the types of the .ts file it imports
// (Node 22.18+; the flag keeps older 22.x working).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import ts from "typescript";
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

// Drive the actual gate and prompt functions, not just the decision table: a status failure must
// reach authentication, and a cancelled prompt must leave the gate pending until a successful retry.
const main = ts.createSourceFile("main.ts", readFileSync(new URL("../src/main.ts", import.meta.url), "utf8"), ts.ScriptTarget.Latest, true);
const names = ["authOpts", "runLock", "biometricGate"];
const functions = main.statements.filter((n) => ts.isFunctionDeclaration(n) && names.includes(n.name?.text));
assert.equal(functions.length, names.length);
const js = ts.transpileModule(functions.map((f) => f.getText(main)).join("\n"), {
  compilerOptions: { target: ts.ScriptTarget.ES2020 },
}).outputText;
for (const unavailable of ["statusError", "biometryLockout"]) {
  const attempts = [];
  const storage = new Map([["pref", "on"]]);
  const context = vm.createContext({
    BIO_KEY: "pref", localStorage: { getItem: (k) => storage.get(k) ?? null, setItem: (k, v) => storage.set(k, v) },
    window: { __TAURI_INTERNALS__: {} }, BiometryType: { FaceID: 2, TouchID: 1 },
    checkStatus: async () => {
      if (unavailable === "statusError") throw new Error("native status unavailable");
      return { isAvailable: false, errorCode: "biometryLockout" };
    },
    authenticate: (reason, options) => new Promise((resolve, reject) => attempts.push({ reason, options, resolve, reject })),
    decidirTrava, semCredencialNoAparelho, showLock() {}, runOptIn() { throw new Error("an enabled lock must not offer opt-in"); },
    lockEl: { hidden: false }, lockMsgEl: { textContent: "" }, lockSecondaryEl: { hidden: true }, lockPrimaryEl: {},
  });
  vm.runInContext(js, context);
  let settled = false;
  const gate = vm.runInContext("biometricGate()", context).then((value) => { settled = true; return value; });
  const tick = () => new Promise((resolve) => setImmediate(resolve));
  await tick();
  assert.equal(attempts.length, 1, unavailable);
  assert.equal(attempts[0].options.allowDeviceCredential, true);
  assert.equal(settled, false, "the gate waits for authentication");
  attempts[0].reject({ code: "userCancel" });
  await tick();
  assert.equal(settled, false, "cancelling cannot unlock the gate");
  assert.equal(context.lockEl.hidden, false);
  context.lockPrimaryEl.onclick();
  assert.equal(attempts.length, 2);
  attempts[1].resolve();
  assert.equal(await gate, true, "a successful retry keeps the biometric preference enabled");
  assert.equal(context.lockEl.hidden, true);
}
console.log(`✓ trava biométrica: ${casos.length + 7} casos`);

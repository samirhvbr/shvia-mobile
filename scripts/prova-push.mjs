#!/usr/bin/env node
// Exercise the same delivery function included by Rust, with the URL at evaluation time.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(new URL("../src-tauri/src/push_token.js", import.meta.url), "utf8");
const hosts = ["ai.shvia.org", "ia.shvia.org", "ia.blue3.com.br"];
const cases = [
  ["https://ai.shvia.org/chat", true],
  ["https://ia.shvia.org/", true],
  ["https://ia.blue3.com.br/", true],
  ["https://github.com/login/oauth/authorize", false],
  ["https://ai.shvia.org.evil.test/", false],
  ["http://ai.shvia.org/chat", false],
  ["tauri://localhost/index.html", false],
];
for (const [url, allowed] of cases) {
  const events = [];
  const context = { location: new URL(url), window: { dispatchEvent: (e) => events.push(e.type) }, Event };
  const deliver = vm.runInNewContext(`(${source})`, context);
  deliver("aB012f", hosts);
  assert.equal(context.window.__shviaPushToken, allowed ? "aB012f" : undefined, url);
  assert.equal(context.window.__shviaPushPlatform, allowed ? "ios" : undefined, url);
  assert.equal(events.join(","), allowed ? "shvia:push-token" : "", url);
}
// The native check saw ShvIA; navigation completes before the queued script executes.
const context = { location: new URL("https://ai.shvia.org/chat"), window: { dispatchEvent() {} }, Event };
const deliver = vm.runInNewContext(`(${source})`, context);
context.location = new URL("https://github.com/login");
deliver("aB012f", hosts);
assert.equal(context.window.__shviaPushToken, undefined, "navigation before eval must not expose the token");
context.location = new URL("https://ai.shvia.org/chat");
deliver("aB012f", hosts);
assert.equal(context.window.__shviaPushToken, "aB012f", "the stored token is delivered on the next ShvIA load");
const hostile = vm.createContext({ location: new URL("https://github.com/login"), window: { dispatchEvent() {} }, Event });
vm.runInContext("Array.prototype.includes = () => true; Array.prototype.indexOf = () => 0;", hostile);
vm.runInContext(`(${source})("aB012f", ${JSON.stringify(hosts)})`, hostile);
assert.equal(hostile.window.__shviaPushToken, undefined, "a page cannot bypass the origin fence by replacing builtins");
console.log(`✓ push token: ${cases.length + 3} cases`);

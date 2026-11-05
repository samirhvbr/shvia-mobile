function deliverPushToken(token, hosts) {
  // Navigation can finish after the native URL check but before eval runs.
  if (location.protocol !== "https:") return;
  // A page can replace Array.prototype methods; the origin fence uses only comparisons.
  let allowed = false;
  for (let i = 0; i < hosts.length; i++) {
    if (hosts[i] === location.hostname) allowed = true;
  }
  if (!allowed) return;
  window.__shviaPushPlatform = "ios";
  window.__shviaPushToken = token;
  window.dispatchEvent(new Event("shvia:push-token"));
}

/**
 * What the biometric gate does on a cold start or a relock, as a pure decision (27/10/2026).
 *
 * Until 0.7.12 the gate let the app through whenever the status query failed or reported biometrics
 * unavailable, even with the lock turned on. On iOS, "unavailable" includes a lockout: someone holding
 * an unlocked phone who fails Face ID five times locks biometrics device-wide, and the next launch of
 * ShvIA opened with no prompt (external review, 26/10/2026; docs/current-state.md listed it as a
 * limitation). The device's own passcode was always available as the fallback (`allowDeviceCredential`),
 * so failing closed does not lock the owner out.
 *
 * The one case with no lock possible is a device with no passcode at all: iOS allows no Face ID without
 * one. There the app lets the person in. That is the owner's decision of 27/10/2026, and the same answer
 * as before for that device.
 */

/** `seguir` = go to ShvIA; `oferecer` = offer to turn the lock on; `travar` = ask before going on. */
export type Decisao = "seguir" | "oferecer" | "travar";

export interface Leitura {
  /** Running inside the Tauri shell. Outside it (the dev server in a browser) there is nothing to lock. */
  emTauri: boolean;
  /** localStorage: "on" | "off" | null (never asked). */
  pref: string | null;
  /** The plugin's status, when the query answered. */
  status?: { isAvailable: boolean; errorCode?: string };
  /** The status query threw. */
  statusFalhou: boolean;
}

export function decidirTrava(l: Leitura): Decisao {
  if (l.pref === "off") return "seguir";
  if (l.statusFalhou || !l.status) {
    // Inside the shell a failed query is not "no biometrics": with the lock on, ask anyway.
    return l.emTauri && l.pref === "on" ? "travar" : "seguir";
  }
  if (l.status.isAvailable) return l.pref === null ? "oferecer" : "travar";
  // Biometrics unavailable right now (lockout, not enrolled, no passcode). Never offered on such a
  // device; with the lock already on, the passcode stands in — unless there is no passcode.
  if (l.pref !== "on") return "seguir";
  return l.status.errorCode === "passcodeNotSet" ? "seguir" : "travar";
}

/**
 * The authentication failed because the device has no passcode or screen lock at all (iOS
 * `passcodeNotSet`, Android `noDeviceCredential`). Nothing can unlock the gate then, so it lets the
 * person in instead of trapping them on the lock screen.
 */
export function semCredencialNoAparelho(erro: unknown): boolean {
  const codigos = ["passcodeNotSet", "noDeviceCredential"];
  if (typeof erro === "string") return codigos.some((c) => erro.includes(c));
  if (erro && typeof erro === "object") {
    const codigo = (erro as { code?: unknown }).code;
    if (typeof codigo === "string" && codigos.includes(codigo)) return true;
    const mensagem = (erro as { message?: unknown }).message;
    return typeof mensagem === "string" && codigos.some((c) => mensagem.includes(c));
  }
  return false;
}

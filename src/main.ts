// Casca de bootstrap do ShvIA Mobile.
//
// A janela Tauri abre esta casca local (instantânea, sem rede) com um splash da
// marca e navega para o ShvIA hospedado. Diferente do desktop, aqui a casca
// VERIFICA o servidor antes de navegar: rede de celular falha o tempo todo, e
// navegar às cegas estampa a tela de erro nativa do WebView. Fluxo:
//   1. ping barato no /up (health nativo do Laravel, ~30 ms; no-cors: resposta
//      opaca serve — só queremos saber se o servidor está alcançável).
//      NÃO usar /api/v1/health aqui: ele sonda os provedores de IA em série e
//      em cache frio passa de 6 s → falso "offline" (visto em produção 14/07);
//   2. alcançável → location.replace() (o splash não fica no histórico);
//   3. falhou → estado "offline" com "Tentar agora" (e "Abrir mesmo assim"
//      como escape, caso o ping falhe por outra razão que não a rede).
//
// Offline com AUTO-RETRY: além do botão, a casca fica tentando sozinha a cada
// 5 s e no evento `online` do navegador — voltou a rede, entra sem clique
// (pedido do Samir, 07/07: "ficou ali para eu apertar o tentar novamente").
//
// Dev: "?hold" congela no estado connecting e "?hold=offline" mostra o estado
// offline — pra estilizar o splash sem ser redirecionado (sem auto-retry).

import { version } from "../package.json";
import { invoke } from "@tauri-apps/api/core";
import {
  authenticate,
  checkStatus,
  BiometryType,
  type Status,
  type AuthOptions,
} from "@tauri-apps/plugin-biometric";

// URL do servidor ShvIA (fonte da verdade).
// Domínio próprio desde 26/07/2026. O legado ia.blue3.com.br serve a MESMA
// instância (mesmo IP) e segue aceito como navegação interna (SERVER_HOSTS em
// src-tauri/src/lib.rs), mas quem o app ABRE é este. Paridade com SHVIA-DESKTOP.
const SHVIA_URL = "https://ai.shvia.org";
// Intervalo do auto-retry no estado offline.
const AUTO_RETRY_MS = 5_000;

const rootEl = document.getElementById("bootstrap")!;
const statusEl = document.getElementById("status")!;
const retryEl = document.getElementById("retry");
const forceEl = document.getElementById("force-open");

// "?hold" = modo de estilização: nenhum timer/navegação automática.
const holdMode = new URLSearchParams(window.location.search).has("hold");

// ── Trava biométrica (M3/ADR-002) ─────────────────────────────────────────
// Gate LOCAL de Face ID/Touch ID antes de navegar pro ShvIA remoto. Roda só
// aqui, na página local — a única com bridge nativo (a página remota NÃO recebe
// comando, ADR-001). Não substitui o cookie de sessão same-origin: é
// conveniência/segurança LOCAL, espelhando o BLUE3-INTRANET-MOBILE.
const BIO_KEY = "shvia_biometric"; // localStorage: "on" | "off" | (ausente = 1ª vez)

const lockEl = document.getElementById("lock") as HTMLElement;
const lockTitleEl = document.getElementById("lock-title")!;
const lockMsgEl = document.getElementById("lock-msg")!;
const lockPrimaryEl = document.getElementById("lock-primary") as HTMLButtonElement;
const lockSecondaryEl = document.getElementById("lock-secondary") as HTMLButtonElement;

function showLock(
  mode: "optin" | "lock",
  o: { title: string; msg: string; primary: string; secondary: string; hideSecondary?: boolean },
): void {
  lockEl.dataset.mode = mode;
  lockTitleEl.textContent = o.title;
  lockMsgEl.textContent = o.msg;
  lockPrimaryEl.textContent = o.primary;
  lockSecondaryEl.textContent = o.secondary;
  lockSecondaryEl.hidden = o.hideSecondary ?? false;
  lockEl.hidden = false;
}

function authOpts(name: string): AuthOptions {
  return {
    allowDeviceCredential: true, // cai no passcode do aparelho se a biometria falhar
    fallbackTitle: "Usar senha do aparelho",
    title: "ShvIA", // Android
    subtitle: `Desbloqueio por ${name}`,
    confirmationRequired: false,
  };
}

// 1ª execução: oferece ativar. "Ativar" confirma com uma autenticação real
// (garante que está enrolado no aparelho) antes de persistir "on".
function runOptIn(name: string): Promise<void> {
  return new Promise((resolve) => {
    showLock("optin", {
      title: `Proteger o ShvIA com ${name}?`,
      msg: `Desbloqueie o app com ${name}, sem digitar a senha a cada acesso. Dá pra mudar depois.`,
      primary: `Ativar ${name}`,
      secondary: "Agora não",
    });
    lockPrimaryEl.onclick = async () => {
      try {
        await authenticate(`Confirme para ativar o ${name}`, authOpts(name));
        localStorage.setItem(BIO_KEY, "on");
        lockEl.hidden = true;
        resolve();
      } catch {
        lockMsgEl.textContent = `Não deu pra confirmar o ${name}. Tente de novo, ou siga sem bloqueio.`;
      }
    };
    lockSecondaryEl.onclick = () => {
      localStorage.setItem(BIO_KEY, "off");
      lockEl.hidden = true;
      resolve();
    };
  });
}

// Cold-start com trava ativa: auto-prompta; se falhar, revela "Desativar
// bloqueio" (que TAMBÉM exige autenticar — senão qualquer um desligaria a trava).
function runLock(name: string): Promise<void> {
  return new Promise((resolve) => {
    const tryUnlock = async () => {
      try {
        await authenticate(`Desbloqueie o ShvIA com ${name}`, authOpts(name));
        lockEl.hidden = true;
        resolve();
      } catch {
        lockMsgEl.textContent = "Não desbloqueou. Toque em Desbloquear pra tentar de novo.";
        lockSecondaryEl.hidden = false;
      }
    };
    showLock("lock", {
      title: "ShvIA bloqueado",
      msg: `Use o ${name} para desbloquear.`,
      primary: "Desbloquear",
      secondary: "Desativar bloqueio",
      hideSecondary: true,
    });
    lockPrimaryEl.onclick = () => void tryUnlock();
    lockSecondaryEl.onclick = async () => {
      try {
        await authenticate("Confirme para desativar o bloqueio", authOpts(name));
        localStorage.setItem(BIO_KEY, "off");
        lockEl.hidden = true;
        resolve();
      } catch {
        lockMsgEl.textContent = "Não foi possível desativar. Toque em Desbloquear.";
      }
    };
    void tryUnlock(); // padrão nativo: prompta assim que a tela abre
  });
}

// Resolve quando pode seguir pro ShvIA: destravado, OU biometria off/ausente no
// aparelho. Com a trava ativa, nunca resolve sem auth — em falha re-prompta (fica
// na tela), então proceedToShvia só navega no resolve.
// Resolves to whether the lock is ACTIVE afterwards (on, and the device has
// biometrics): the native side needs it to relock when the app comes back.
async function biometricGate(): Promise<boolean> {
  const pref = localStorage.getItem(BIO_KEY);
  if (pref === "off") return false;

  let status: Status;
  try {
    status = await checkStatus();
  } catch {
    return false; // sem bridge nativo (dev no navegador/desktop) → não trava
  }
  if (!status.isAvailable) return false; // aparelho sem biometria → não trava

  const name =
    status.biometryType === BiometryType.FaceID
      ? "Face ID"
      : status.biometryType === BiometryType.TouchID
        ? "Touch ID"
        : "biometria";

  if (pref === null) {
    await runOptIn(name);
  } else {
    await runLock(name); // pref === "on"
  }
  // "Agora não" and "Desativar bloqueio" both leave "off" behind.
  return localStorage.getItem(BIO_KEY) === "on";
}

// Relock (answer of 24/09/2026: "na hora"). When the app comes back from the
// background with the lock on, the native side sends the webview here with
// `?relock=<path of the page it was on>` (src-tauri/src/lib.rs, `retravar`).
// Only a path is accepted, never a host: the gate must not become a redirect.
function destino(): string {
  const volta = new URLSearchParams(window.location.search).get("relock");
  const segura =
    volta !== null && volta.startsWith("/") && !volta.startsWith("//") && !volta.includes("\\");
  return segura ? SHVIA_URL + volta : SHVIA_URL;
}

let navigating = false;
// Navegação única pro ShvIA, atrás do gate biométrico. Guard: se o auto-retry e
// o evento `online` dispararem juntos, só o primeiro gateia/navega.
async function proceedToShvia(): Promise<void> {
  if (navigating) return;
  navigating = true;
  const travada = await biometricGate();
  // Tell the native side, which relocks on the next return to the app only when
  // this is true. No bridge (browser dev) = nothing to tell.
  invoke("trava_biometrica", { ligada: travada }).catch(() => {});
  window.location.replace(destino());
}

let autoRetryTimer: number | undefined;
let checking = false;

function scheduleAutoRetry(): void {
  window.clearTimeout(autoRetryTimer);
  if (holdMode) return;
  autoRetryTimer = window.setTimeout(() => {
    void autoCheck();
  }, AUTO_RETRY_MS);
}

function setState(state: "connecting" | "offline"): void {
  rootEl.dataset.state = state;
  statusEl.textContent =
    state === "connecting"
      ? "Conectando ao ShvIA…"
      : "Sem conexão com o servidor — reconectando…";
  window.clearTimeout(autoRetryTimer);
  if (state === "offline") {
    scheduleAutoRetry();
  }
}

async function serverReachable(timeoutMs = 6000): Promise<boolean> {
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), timeoutMs);
  try {
    await fetch(`${SHVIA_URL}/up`, {
      mode: "no-cors",
      cache: "no-store",
      signal: ctl.signal,
    });
    return true;
  } catch {
    return false;
  } finally {
    clearTimeout(timer);
  }
}

// Tentativa silenciosa (auto-retry): não mexe na UI enquanto verifica — só
// navega quando o servidor voltar. Sem sobreposição: a próxima só é agendada
// quando esta termina (e só se ainda estivermos offline).
async function autoCheck(): Promise<void> {
  if (checking) return;
  checking = true;
  const ok = await serverReachable();
  checking = false;
  if (ok) {
    void proceedToShvia();
    return;
  }
  if (rootEl.dataset.state === "offline") {
    scheduleAutoRetry();
  }
}

// Tentativa manual/inicial: mostra o estado "connecting" (sonar) enquanto tenta.
async function connect(): Promise<void> {
  setState("connecting");
  if (await serverReachable()) {
    void proceedToShvia();
    return;
  }
  setState("offline");
}

window.addEventListener("DOMContentLoaded", () => {
  // Versão da casca visível na base (vem de package.json, que o
  // scripts/sync-version.mjs mantém igual ao version.md).
  const footEl = document.getElementById("foot");
  // Só a versão: o splash é PRÉ-LOGIN e a marca Blue3 só pode aparecer depois do
  // login, para e-mail dela (SHVIA-WEB/config/brand.php). O <h1>ShvIA</h1> logo
  // acima já nomeia o produto; aqui o que interessa é saber QUAL build está no
  // aparelho — a pergunta de todo teste via TestFlight.
  if (footEl) footEl.textContent = `v${version}`;

  retryEl?.addEventListener("click", () => {
    void connect();
  });
  forceEl?.addEventListener("click", () => {
    void proceedToShvia();
  });
  // Rede voltou (evento do SO/navegador) → tenta na hora, sem esperar os 5 s.
  window.addEventListener("online", () => {
    if (!holdMode && rootEl.dataset.state === "offline") {
      window.clearTimeout(autoRetryTimer);
      void autoCheck();
    }
  });

  if (holdMode) {
    const hold = new URLSearchParams(window.location.search).get("hold");
    setState(hold === "offline" ? "offline" : "connecting");
    return;
  }
  void connect();
});

export {};

//! ShvIA Mobile — shell fino Tauri 2 (iOS + Android).
//!
//! A janela abre a casca local (`src/`), que mostra um splash com a marca e
//! redireciona o WebView (WKWebView no iOS, System WebView no Android) para o
//! ShvIA hospedado (`https://ai.shvia.org`). A partir daí a UI é o próprio
//! Blade do ShvIA — "mesmas funções" (espelha a postura do SHVIA-DESKTOP).
//!
//! Postura de menor privilégio: **nenhum comando nativo é exposto à página
//! remota** — o servidor é a fonte da verdade; o cliente não abre banco nem
//! guarda segredo.
//!
//! **Mobile-only:** sem menu / multi-janela / geometria de janela (isso é
//! desktop). A leitura em voz (TTS) usa o `speechSynthesis` nativo do WebView
//! (iOS/Android já têm voz pt-BR), então **não** precisa da ponte espeak do
//! Linux/WebKitGTK. Links externos abrem no navegador do SO via `on_navigation`.

use std::sync::Mutex;

use tauri::{webview::PageLoadEvent, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "ios")]
use tauri_plugin_shvia_push::ShviaPushExt;

/// Último device token do APNs (M3/push). Vive em estado da casca para ser
/// REINJETADO em cada page load da página remota — o token pode chegar antes
/// da página (cold start) ou a página pode recarregar depois do token.
struct PushToken(Mutex<Option<String>>);

/// Pede a permissão de notificação UMA vez por execução, e só no primeiro
/// load REMOTO (usuário logado = momento com contexto, não no splash).
#[cfg(target_os = "ios")]
struct PushPermissionAsked(AtomicBool);

/// Face ID lock on or off, as the local shell last reported it (`trava_biometrica`). The
/// preference lives in the local shell's `localStorage`, an origin this process cannot read,
/// so the shell reports it on every start and after every change.
struct TravaLigada(AtomicBool);

/// Origin of the local shell, learned on its first page load. It differs per platform
/// (`tauri://localhost` on iOS, `http://tauri.localhost` on Android), and the relock has to
/// send the webview back there.
struct CascaLocal(Mutex<Option<tauri::Url>>);

/// Called by the local shell only. The remote page has no IPC at all: there is no `remote`
/// capability, so this is not a command the ShvIA page can reach (ADR-001 still holds).
#[tauri::command]
fn trava_biometrica(ligada: bool, trava: tauri::State<'_, TravaLigada>) {
    trava.0.store(ligada, Ordering::SeqCst);
}

/// Where the webview goes when the app comes back with the lock on: the local shell's gate,
/// carrying the page to return to. Only the PATH and query travel, never a host, so the
/// gate cannot become an open redirect. `None` when the current page is not the ShvIA
/// server (the gate itself, the splash, anything else): nothing to lock.
// Only the mobile `Resumed` handler calls these; on the host they are reached by tests.
#[cfg_attr(not(mobile), allow(dead_code))]
fn alvo_da_retrava(casca: &tauri::Url, atual: &tauri::Url) -> Option<String> {
    let host = atual.host_str()?;
    if atual.scheme() != "https" || !SERVER_HOSTS.contains(&host) {
        return None;
    }
    let mut volta = atual.path().to_string();
    if let Some(q) = atual.query() {
        volta.push('?');
        volta.push_str(q);
    }
    let mut alvo = casca.join("index.html").ok()?;
    alvo.query_pairs_mut().clear().append_pair("relock", &volta);
    Some(alvo.to_string())
}

/// The page decides whether to go, because only it knows one thing: a file picker that was
/// just opened. Android runs the picker as another activity, so coming back from it is ALSO
/// a resume, and relocking there would reload the page and throw the chosen file away. The
/// mark lasts 10 minutes and is spent by the first resume, picker or not.
// Only the mobile `Resumed` handler calls these; on the host they are reached by tests.
#[cfg_attr(not(mobile), allow(dead_code))]
fn retrava_js(alvo: &str) -> String {
    format!(
        "(function(){{var t=window.__shviaSeletorEm||0;window.__shviaSeletorEm=0;if(t&&Date.now()-t<600000)return;location.replace({alvo:?});}})();"
    )
}

/// Injected with the offline banner: marks when a file input is opened, for `retrava_js`.
/// A capture listener on `document` sees it because the ShvIA's file inputs live in the DOM
/// (`hidden`) and are opened with `.click()`, which dispatches a real click (checked on
/// 24/09/2026: `attach-input`, `project-files-input`, `cc-anexo-input`). An input opened
/// while detached from the document would slip past; there is none today.
const MARCA_SELETOR_JS: &str = r#"(function () {
  if (window.__shviaSeletor) return;
  window.__shviaSeletor = true;
  document.addEventListener('click', function (e) {
    var el = e.target;
    if (el && el.matches && el.matches('input[type=file]')) window.__shviaSeletorEm = Date.now();
  }, true);
})();"#;

/// Relock on return: runs on every resume of the main window, and does nothing unless the
/// lock is on and the current page is the ShvIA server.
// Only the mobile `Resumed` handler calls these; on the host they are reached by tests.
#[cfg_attr(not(mobile), allow(dead_code))]
fn retravar<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if !app.state::<TravaLigada>().0.load(Ordering::SeqCst) {
        return;
    }
    let Some(casca) = app.state::<CascaLocal>().0.lock().unwrap().clone() else {
        return;
    };
    let Some(janela) = app.get_webview_window("main") else {
        return;
    };
    let Ok(atual) = janela.url() else {
        return;
    };
    if let Some(alvo) = alvo_da_retrava(&casca, &atual) {
        let _ = janela.eval(retrava_js(&alvo));
    }
}

/// JS que entrega o token à página remota. O front do ShvIA web
/// (`registerPushToken` no app.js) escuta o evento e faz o POST /push/token
/// com a sessão. Token validado como hex antes de entrar no JS.
fn push_token_js(token: &str) -> Option<String> {
    if token.is_empty() || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!(
        "window.__shviaPushPlatform='ios';window.__shviaPushToken='{token}';window.dispatchEvent(new Event('shvia:push-token'));"
    ))
}

/// Injetado em cada página carregada (`on_page_load`): uma **tarja "Sistema
/// Offline"** clicável para recarregar. O `eval` do Tauri roda fora da CSP da
/// página, então funciona na página remota do ShvIA.
///
/// v2 (01/08/2026, espelhando o desktop): `navigator.onLine` vira só GATILHO DE
/// SUSPEITA — a tarja apenas aparece se uma sonda REAL ao `/up` (health barato
/// do Laravel) falhar, e re-sonda a cada 15 s até o servidor voltar. O bug que
/// motivou foi no WebKitGTK do desktop (GNetworkMonitor mente com VPN/rotas
/// incomuns); o WKWebView é mais confiável, mas as cascas são espelhadas e a
/// sonda é estritamente melhor nos dois.
const OFFLINE_BANNER_JS: &str = r#"(function () {
  if (window.__shviaOffline) return;
  window.__shviaOffline = true;
  var bar = document.createElement('div');
  bar.id = 'shvia-offline-bar';
  bar.textContent = '⚠  Sistema Offline — toque para recarregar';
  var s = bar.style;
  s.position='fixed'; s.top='0'; s.left='0'; s.right='0'; s.zIndex='2147483647';
  s.background='#c0392b'; s.color='#fff'; s.textAlign='center'; s.padding='calc(env(safe-area-inset-top,0px) + 8px) 12px 8px';
  s.font='600 14px system-ui,sans-serif'; s.letterSpacing='.02em'; s.cursor='pointer';
  s.boxShadow='0 2px 8px rgba(0,0,0,.35)';
  bar.addEventListener('click', function () { location.reload(); });
  function mount(){ var r=document.body||document.documentElement; if(r&&!document.getElementById('shvia-offline-bar')) r.appendChild(bar); }
  var checking=false, timer=null;
  function show(){ bar.style.display='block'; if(!timer) timer=setInterval(check, 15000); }
  function hide(){ bar.style.display='none'; if(timer){ clearInterval(timer); timer=null; } }
  function check(){
    if (checking) return; checking = true;
    var ctl = ('AbortController' in window) ? new AbortController() : null;
    var to = setTimeout(function(){ if (ctl) ctl.abort(); }, 5000);
    fetch('/up', { cache:'no-store', signal: ctl && ctl.signal })
      .then(function(r){ if (r.ok) { hide(); } else { show(); } })
      .catch(function(){ show(); })
      .finally(function(){ clearTimeout(to); checking = false; });
  }
  function update(){ if (navigator.onLine) { hide(); } else { check(); } }
  mount(); update();
  window.addEventListener('online', update);
  window.addEventListener('offline', update);
})();"#;

/// Host CANÔNICO do servidor do ShvIA — o destino da navegação da casca.
const SERVER_HOST: &str = "ai.shvia.org";

/// Hosts EXATOS aceitos como o servidor do ShvIA. Mantido em PARIDADE com
/// `SERVER_HOSTS` de SHVIA-DESKTOP/src-tauri/src/lib.rs — as duas cascas são
/// espelhadas de propósito.
///
/// Lista exata, **sem sufixo curinga**. O curinga anterior
/// (`host.ends_with(".blue3.com.br")`) aceitava QUALQUER subdomínio do domínio
/// corporativo dentro do app; o desktop fechou isso no 0.9.0 e o mobile herda a
/// mesma postura agora. Cada entrada foi verificada por DNS:
///
/// - `ai.shvia.org` — canônico (200.36.196.254).
/// - `ia.shvia.org` — CNAME de `ai.shvia.org`, mesma instância.
/// - `ia.blue3.com.br` — domínio legado, MESMO IP. Só durante a transição;
///   remover esta linha é tudo o que o desligamento dele exige.
///
/// O ápex `shvia.org` fica FORA de propósito: resolve para outro IP
/// (170.233.231.20) e serve a landing, não o app — tem de abrir no navegador.
///
/// No MESMO IP do ápex vive `mem.shvia.org` (servidor ai-memory, desde
/// 31/07/2026) — o melhor argumento contra o curinga que esta lista recusa: é
/// `.shvia.org`, é nosso, é legítimo, e mesmo assim **não pode entrar**. Serve
/// wiki markdown **escrita por agentes**, e o curinga o teria admitido sozinho
/// no dia em que o DNS subiu, sem ninguém decidir nada.
const SERVER_HOSTS: &[&str] = &[SERVER_HOST, "ia.shvia.org", "ia.blue3.com.br"];

/// GitHub's sign-in and OAuth consent pages: the only part of github.com that stays inside
/// the app (finding f169, the owner's answer on 24/09/2026: "it is on, fix it in the app").
/// The web's "connect GitHub" goes to `github.com/login/oauth/authorize` and comes back to
/// `ai.shvia.org/integracoes/github/callback`. Opened in the OS browser, like every external
/// link, the callback landed where the app's session is not, and the user never came back to
/// the app. Inside the webview, the whole round trip keeps the app's cookie jar.
///
/// By PATH, not by host: `/login` (sign-in and `/login/oauth/…`) and `/session`, `/sessions/…`
/// (the sign-in form's POST and the two-factor steps). Everything else on github.com, a
/// repository or the sign-up page, is still an external link. These pages get no IPC and no
/// injected script (`recebe_script_da_casca`). A sign-in that leaves GitHub (SSO through
/// another provider) opens outside and does not come back; password and 2FA do.
fn e_login_do_github(url: &tauri::Url) -> bool {
    if url.scheme() != "https" || url.host_str() != Some("github.com") {
        return false;
    }
    let p = url.path();
    p == "/login" || p.starts_with("/login/") || p == "/session" || p.starts_with("/sessions/")
}

/// Which pages get the shell's script (version, offline banner, picker mark, push token):
/// the ShvIA server only. It used to be "every page that is not the local shell", which was
/// the same thing while nothing else loaded inside the app. With GitHub's sign-in inside it,
/// that rule would hand the push token to github.com.
fn recebe_script_da_casca(url: &tauri::Url) -> bool {
    url.scheme() == "https" && url.host_str().is_some_and(|h| SERVER_HOSTS.contains(&h))
}

/// Uma navegação fica **no app** se for a casca local (localhost/tauri) ou um
/// host do servidor do ShvIA em https; qualquer outra origem é link externo e
/// abre no navegador do SO.
fn is_internal(url: &tauri::Url) -> bool {
    let host = match url.host_str() {
        Some(h) => h,
        None => return false,
    };
    // Casca local: `http` (Vite dev + prod Android) e `tauri` (prod iOS).
    // O servidor exige https.
    match url.scheme() {
        "https" => SERVER_HOSTS.contains(&host),
        "http" => host == "localhost" || host == "tauri.localhost",
        "tauri" => host == "localhost",
        _ => false,
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)] // no host o bloco `#[cfg(mobile)]` some → `mut` não usado
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    // Biometria (M3/ADR-002): plugin **mobile-only** — o crate é `#![cfg(mobile)]`
    // e nem existe no host (por isso o registro fica atrás de `#[cfg(mobile)]`).
    // Expõe `authenticate`/`status` à página LOCAL (`src/`), que faz o gate Face
    // ID antes de navegar pro ShvIA remoto. A página remota segue sem acesso nativo.
    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_biometric::init());
    }

    // Push (M3/ADR-002): plugin interno iOS-only. O token do APNs sobe do
    // Swift por Channel e é injetado na página remota; a página continua sem
    // NENHUM comando nativo exposto (quem fala com o plugin é o Rust daqui).
    // Android (FCM) fica para depois — por isso `target_os = "ios"`, não `mobile`.
    #[cfg(target_os = "ios")]
    {
        builder = builder.plugin(tauri_plugin_shvia_push::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![trava_biometrica])
        .on_window_event(|janela, evento| {
            // Mobile only: `Resumed` is Android's `onResume` and iOS's
            // `applicationWillEnterForeground` (the Face ID prompt itself does not fire it).
            #[cfg(mobile)]
            {
                if let tauri::WindowEvent::Resumed = evento {
                    retravar(janela.app_handle());
                }
            }
            #[cfg(not(mobile))]
            {
                let _ = (janela, evento);
            }
        })
        .setup(|app| {
            app.manage(PushToken(Mutex::new(None)));
            app.manage(TravaLigada(AtomicBool::new(false)));
            app.manage(CascaLocal(Mutex::new(None)));
            #[cfg(target_os = "ios")]
            app.manage(PushPermissionAsked(AtomicBool::new(false)));

            let handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("ShvIA")
                .on_navigation(move |url| {
                    if is_internal(url) || e_login_do_github(url) {
                        return true;
                    }
                    // link externo → navegador do SO, não dentro do app.
                    let _ = handle.opener().open_url(url.to_string(), None::<&str>);
                    false
                })
                // injeta a tarja "Sistema Offline" SÓ nas páginas remotas do
                // ShvIA: a casca local tem UI própria de offline (splash com
                // "Tentar novamente") — com a tarja junto ficava em dobro.
                // Também expõe a versão da casca (`window.__shviaShellVersion`)
                // para o ShvIA web poder exibi-la junto da versão do servidor.
                .on_page_load(|webview, payload| {
                    if let PageLoadEvent::Finished = payload.event() {
                        let host = payload.url().host_str().unwrap_or_default().to_owned();
                        if host == "localhost" || host == "tauri.localhost" {
                            // Learn where the local shell lives, for the relock.
                            let mut casca = payload.url().clone();
                            casca.set_path("/");
                            casca.set_query(None);
                            casca.set_fragment(None);
                            *webview.state::<CascaLocal>().0.lock().unwrap() = Some(casca);
                        } else if recebe_script_da_casca(payload.url()) {
                            let mut js = format!(
                                "window.__shviaShellVersion={:?};{}{}",
                                env!("CARGO_PKG_VERSION"),
                                OFFLINE_BANNER_JS,
                                MARCA_SELETOR_JS
                            );
                            // Token de push conhecido? Reinjetar a cada load —
                            // o front (registerPushToken) é idempotente.
                            if let Some(tok) =
                                webview.state::<PushToken>().0.lock().unwrap().clone()
                            {
                                if let Some(tjs) = push_token_js(&tok) {
                                    js.push_str(&tjs);
                                }
                            }
                            let _ = webview.eval(js);

                            // 1º load remoto = usuário chegou ao ShvIA logado:
                            // hora de pedir a permissão de notificação (com
                            // contexto, não no splash). `requestPermission`
                            // BLOQUEIA até o usuário decidir → thread própria.
                            #[cfg(target_os = "ios")]
                            {
                                let asked = webview.state::<PushPermissionAsked>();
                                if !asked.0.swap(true, Ordering::SeqCst) {
                                    let wv = webview.clone();
                                    std::thread::spawn(move || {
                                        let _ = wv.shvia_push().request_permission();
                                    });
                                }
                            }
                        }
                    }
                })
                .build()?;

            // Canal de eventos do push (iOS): o Swift entrega token/rota/erro
            // aqui; token vai pro estado + eval; tap navega (rota sanitizada).
            #[cfg(target_os = "ios")]
            {
                let handle = app.handle().clone();
                let channel = tauri::ipc::Channel::new(move |event| {
                    let tauri::ipc::InvokeResponseBody::Json(json) = event else {
                        return Ok(());
                    };
                    let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) else {
                        return Ok(());
                    };
                    match v.get("type").and_then(|t| t.as_str()) {
                        Some("token") => {
                            if let Some(tok) = v.get("token").and_then(|t| t.as_str()) {
                                if let Some(js) = push_token_js(tok) {
                                    *handle.state::<PushToken>().0.lock().unwrap() =
                                        Some(tok.to_string());
                                    if let Some(w) = handle.get_webview_window("main") {
                                        let _ = w.eval(js);
                                    }
                                }
                            }
                        }
                        Some("route") => {
                            // Tap em notificação. Só caminho relativo simples:
                            // nada de URL absoluta/esquema/escape — a rota vem
                            // do payload do push e não é confiável por padrão.
                            if let Some(route) = v.get("route").and_then(|r| r.as_str()) {
                                let segura = route.starts_with('/')
                                    && !route.starts_with("//")
                                    && route.chars().all(|c| {
                                        c.is_ascii_graphic()
                                            && !matches!(c, '"' | '\'' | '\\' | '<' | '>' | '`')
                                    });
                                if segura {
                                    if let Some(w) = handle.get_webview_window("main") {
                                        let _ = w.eval(format!("location.assign({route:?})"));
                                    }
                                }
                            }
                        }
                        Some("error") => {
                            // Sem token não há push; o app segue normal.
                            eprintln!(
                                "shvia-push: {}",
                                v.get("message").and_then(|m| m.as_str()).unwrap_or("erro")
                            );
                        }
                        _ => {}
                    }
                    Ok(())
                });
                let _ = app.shvia_push().watch_token(channel);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao executar o aplicativo ShvIA Mobile");
}

// O `mod tests` fica no FIM do arquivo, e não no meio: o lint
// `clippy::items_after_test_module` reprova qualquer item declarado depois dele, e o
// `run()` estava logo abaixo — `cargo clippy -- -D warnings` deste repo já vinha
// VERMELHO antes de 02/09/2026, sem ninguém ver, porque o CI daqui não roda clippy.
// Mesma família do achado F-20 (que mediu só o DESKTOP).
#[cfg(test)]
mod tests {
    use super::{alvo_da_retrava, e_login_do_github, is_internal, recebe_script_da_casca, retrava_js};

    /// GitHub's sign-in, consent and two-factor steps stay in the app, so the OAuth round
    /// trip comes back with the app's session.
    #[test]
    fn o_login_do_github_fica_no_app() {
        for u in [
            "https://github.com/login",
            "https://github.com/login?return_to=%2Flogin%2Foauth%2Fauthorize",
            "https://github.com/login/oauth/authorize?client_id=x&state=y",
            "https://github.com/session",
            "https://github.com/sessions/two-factor",
            "https://github.com/sessions/verified-device",
        ] {
            assert!(e_login_do_github(&url(u)), "{u} deveria ficar no app");
        }
    }

    /// The rest of github.com, and anything that only looks like it, stays external.
    #[test]
    fn o_resto_do_github_continua_externo() {
        for u in [
            "https://github.com/samirhvbr/shvia-web",
            "https://github.com/signup",
            "https://github.com/login-evil",
            "https://github.com/sessionsX",
            "http://github.com/login",
            "https://gist.github.com/login",
            "https://github.com.evil.com/login",
        ] {
            assert!(!e_login_do_github(&url(u)), "{u} deveria abrir fora");
        }
    }

    /// The shell's script (and the push token in it) goes to the ShvIA server only, never to
    /// the GitHub pages that now load inside the app, and never to the local shell.
    ///
    /// The helper alone proves nothing if `on_page_load` stops calling it, and a closure
    /// inside `run()` cannot be driven without a device. So this is a SOURCE check, declared
    /// as one: the injection branch must be guarded by the helper.
    ///
    /// Only the code ABOVE this module counts: the file as a whole always contains the
    /// strings below, in these very asserts, and a first version of this test passed with the
    /// guard removed for exactly that reason.
    #[test]
    fn a_injecao_e_a_navegacao_passam_pelos_filtros() {
        let codigo = include_str!("lib.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("o módulo de testes fica no fim do arquivo");
        assert!(codigo.contains("} else if recebe_script_da_casca(payload.url()) {"));
        assert!(codigo.contains("if is_internal(url) || e_login_do_github(url) {"));
    }

    #[test]
    fn so_o_servidor_recebe_o_script_da_casca() {
        assert!(recebe_script_da_casca(&url("https://ai.shvia.org/chat")));
        assert!(recebe_script_da_casca(&url("https://ia.blue3.com.br/")));
        assert!(!recebe_script_da_casca(&url("https://github.com/login/oauth/authorize")));
        assert!(!recebe_script_da_casca(&url("http://ai.shvia.org/")));
        assert!(!recebe_script_da_casca(&url("tauri://localhost/index.html")));
    }

    fn url(u: &str) -> tauri::Url {
        u.parse().expect("url de teste válida")
    }

    /// The relock sends the webview to the local gate on each platform, carrying only the
    /// PATH and query of the page it was on. The fragment and the host stay behind.
    #[test]
    fn a_retrava_volta_so_pelo_caminho() {
        for casca in ["tauri://localhost/", "http://tauri.localhost/"] {
            let alvo = alvo_da_retrava(&url(casca), &url("https://ai.shvia.org/chat/42?modo=code#fim"))
                .expect("a página do servidor retrava");
            let alvo = url(&alvo);
            assert_eq!(alvo.scheme(), url(casca).scheme());
            assert_eq!(alvo.host_str(), url(casca).host_str());
            assert_eq!(alvo.path(), "/index.html");
            let pares: Vec<(String, String)> =
                alvo.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
            assert_eq!(pares, vec![("relock".to_string(), "/chat/42?modo=code".to_string())]);
        }
    }

    /// Only a page of the ShvIA server relocks. The gate itself must not (it would loop),
    /// and neither must anything that is not the server.
    #[test]
    fn so_a_pagina_do_servidor_retrava() {
        let casca = url("tauri://localhost/");
        assert!(alvo_da_retrava(&casca, &url("tauri://localhost/index.html?relock=%2F")).is_none());
        assert!(alvo_da_retrava(&casca, &url("http://tauri.localhost/index.html")).is_none());
        assert!(alvo_da_retrava(&casca, &url("http://ai.shvia.org/chat")).is_none());
        assert!(alvo_da_retrava(&casca, &url("https://mem.shvia.org/")).is_none());
        assert!(alvo_da_retrava(&casca, &url("https://ai.shvia.org.evil.com/")).is_none());
        assert!(alvo_da_retrava(&casca, &url("https://ia.blue3.com.br/x")).is_some());
    }

    /// The injected JS spares a file picker opened in the last 10 minutes, spends the mark
    /// either way, and carries the target as a string literal.
    #[test]
    fn o_js_da_retrava_poupa_o_seletor_e_leva_o_alvo_como_texto() {
        let js = retrava_js("tauri://localhost/index.html?relock=%2Fchat");
        assert!(js.contains("location.replace(\"tauri://localhost/index.html?relock=%2Fchat\")"));
        assert!(js.contains("window.__shviaSeletorEm=0;"));
        assert!(js.contains("<600000)return;"));
    }

    fn internal(url: &str) -> bool {
        is_internal(&url.parse().expect("url de teste válida"))
    }

    /// A casca empacotada tem URL diferente por plataforma. Se qualquer uma
    /// deixar de ser interna, a navegação inicial é bloqueada e o app fica preso
    /// no splash — sem sintoma em `tauri dev` (que usa devUrl).
    #[test]
    fn casca_local_e_interna() {
        assert!(internal("tauri://localhost/index.html"));
        assert!(internal("http://tauri.localhost/index.html"));
        assert!(internal("http://localhost:1420/"));
    }

    /// Dual-host da migração (26/07): as três faces do servidor entram, e o dia
    /// em que o legado sair é só tirar a linha dele de SERVER_HOSTS.
    #[test]
    fn as_tres_faces_do_servidor_sao_internas() {
        assert!(internal("https://ai.shvia.org/chat"));
        assert!(internal("https://ia.shvia.org/chat"));
        assert!(internal("https://ia.blue3.com.br/chat"));
        assert!(!internal("http://ai.shvia.org/chat")); // servidor só em https
    }

    /// Fim do curinga: o ápex shvia.org é a LANDING (outro IP) e subdomínio
    /// qualquer de blue3.com.br não é o ShvIA. Nada disso carrega dentro do app.
    ///
    /// `mem.shvia.org` (ai-memory, mesmo IP do ápex) é o caso REAL, não
    /// hipotético: subdomínio nosso, legítimo, no ar, servindo wiki escrita por
    /// agentes. É o que o curinga deixaria entrar sem ninguém decidir nada.
    #[test]
    fn outras_origens_sao_externas() {
        assert!(!internal("https://shvia.org/"));
        assert!(!internal("https://www.shvia.org/"));
        assert!(!internal("https://mem.shvia.org/"));
        assert!(!internal("https://evil.shvia.org/"));
        assert!(!internal("https://blue3.com.br/"));
        assert!(!internal("https://evil.blue3.com.br/"));
        assert!(!internal("https://ai.shvia.org.evil.com/"));
        assert!(!internal("file:///etc/passwd"));
    }
    /// `AGENTS.md` e `CLAUDE.md` são o mesmo texto abaixo do H1 — achado F-21.
    ///
    /// Os dois arquivos JÁ exigiam isso, por escrito, de si mesmos. E os dois violavam:
    /// no DESKTOP o comentário HTML do topo, no MOBILE o blockquote "Leia também". Não por
    /// desleixo — o bloco que divergia era exatamente o que dizia *"este arquivo é espelho
    /// do outro"*, e escrito em 1ª pessoa ele **não pode** ser idêntico nos dois. A regra
    /// era impossível de cumprir, o que é a razão pela qual instrução sem guarda apodrece:
    /// ninguém percebe que está pedindo o impossível.
    ///
    /// O ponteiro foi reescrito na 3ª pessoa (nomeia os dois arquivos, não "este"), e agora
    /// a régua vale de verdade. Um `diff` de uma linha, que roda a cada `cargo test`.
    #[test]
    fn agents_e_claude_sao_espelho() {
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        // Nos dois repos Tauri o Cargo.toml vive em `src-tauri/`; no CODE, na raiz.
        let base = if raiz.join("../AGENTS.md").exists() { raiz.join("..") } else { raiz.to_path_buf() };
        let ler = |n: &str| {
            let p = base.join(n);
            let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{} ilegível: {e}", p.display()));
            // Fora o H1 (a única linha que PODE diferir: cada arquivo tem seu título).
            t.lines().skip(1).collect::<Vec<_>>().join("\n")
        };
        let agents = ler("AGENTS.md");
        let claude = ler("CLAUDE.md");
        assert!(agents.len() > 500, "AGENTS.md tem {} bytes abaixo do H1 — a régua estaria medindo o vazio", agents.len());
        if agents != claude {
            let a: Vec<&str> = agents.lines().collect();
            let c: Vec<&str> = claude.lines().collect();
            let primeira = (0..a.len().max(c.len()))
                .find(|&i| a.get(i) != c.get(i))
                .map(|i| format!("linha {} abaixo do H1:\n    AGENTS.md: {:?}\n    CLAUDE.md: {:?}",
                    i + 2, a.get(i).unwrap_or(&"<fim>"), c.get(i).unwrap_or(&"<fim>")))
                .unwrap_or_default();
            panic!("AGENTS.md e CLAUDE.md divergem abaixo do H1 — {primeira}");
        }
    }

    /// Toda `uses:` do CI está pinada por SHA de commit — achado F-09.
    ///
    /// Tag de GitHub Action é **ponteiro móvel**: `actions/checkout@v4` roda o que o dono
    /// do repositório publicar amanhã sob aquela tag, com as permissões deste workflow e
    /// acesso ao token do job. Não é hipótese remota — é o vetor de `tj-actions/changed-files`
    /// (03/2025), em que uma tag movida passou a vazar segredos de milhares de repositórios.
    ///
    /// A régua persegue a causa, não o sintoma: o defeito não é "esta action está solta", é
    /// **uma action nova entrar sem pin**. Por isso ela varre o diretório inteiro e não uma
    /// lista — um workflow novo já nasce medido.
    ///
    /// O comentário `# vX.Y.Z` ao lado do SHA não é enfeite: sem ele, subir o pin vira
    /// arqueologia. A régua exige os dois.
    #[test]
    fn toda_action_do_ci_esta_pinada_por_sha() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows");
        let mut vistos = 0usize;
        let mut soltas: Vec<String> = Vec::new();

        let entradas = std::fs::read_dir(&dir).expect("o repositório tem .github/workflows");
        for e in entradas.flatten() {
            let caminho = e.path();
            let ext = caminho.extension().and_then(|x| x.to_str()).unwrap_or("");
            if ext != "yml" && ext != "yaml" {
                continue;
            }
            let arquivo = caminho.file_name().unwrap().to_string_lossy().to_string();
            let texto = std::fs::read_to_string(&caminho).expect("workflow legível");
            for (n, linha) in texto.lines().enumerate() {
                let corte = linha.trim_start();
                // Só `uses:` de action; `uses:` dentro de comentário não conta.
                if corte.starts_with('#') {
                    continue;
                }
                let Some(resto) = corte.strip_prefix("- uses:").or_else(|| corte.strip_prefix("uses:")) else {
                    continue;
                };
                let referencia = resto.trim().split('#').next().unwrap_or("").trim();
                // `uses:` local (`./algo`) e de container (`docker://`) não têm SHA a pinar.
                if referencia.starts_with('.') || referencia.starts_with("docker://") {
                    continue;
                }
                vistos += 1;
                let sha = referencia.rsplit('@').next().unwrap_or("");
                let pinada = sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit());
                // O comentário que diz QUAL versão o SHA é.
                let tem_nota = linha.contains('#');
                if !pinada || !tem_nota {
                    let porque = if pinada { "sem o comentário dizendo a versão" } else { "não é SHA de 40 hex" };
                    soltas.push(format!("{arquivo}:{}  {referencia}  ({porque})", n + 1));
                }
            }
        }

        assert!(vistos > 0, "nenhuma `uses:` encontrada — a régua estaria medindo o vazio");
        assert!(
            soltas.is_empty(),
            "action(s) do CI sem pin por SHA (+ comentário da versão):\n  {}\n\
             Resolva a tag com:\n  \
             gh api repos/<dono>/<repo>/git/ref/tags/<tag> --jq '.object.sha'",
            soltas.join("\n  ")
        );
    }

    /// Todo `.md` de `docs/` é alcançável por link a partir do índice — achado D-DOC-10.
    ///
    /// Documento órfão não é doc velha: é doc que **ninguém sabe que existe**. O
    /// `docs/loja-ficha.md` do MOBILE é a ficha do App Store Connect pronta para colar, com
    /// os limites de caracteres da Apple anotados — escrita em 04/08 e nunca linkada, num
    /// repo cuja submissão está pendente. O custo de um órfão não é o arquivo; é alguém
    /// reescrever o que já estava pronto.
    ///
    /// A régua não julga se a doc está atualizada — julga se dá para CHEGAR nela. Um `.md`
    /// novo em `docs/` deixa o `cargo test` vermelho até alguém decidir onde ele entra no
    /// índice, que é a decisão que ninguém toma quando o arquivo simplesmente aparece.
    /// The Android resources compile. aapt2 refuses the whole resource set when an XML comment
    /// holds a double dash, and the theme comments did ("mesmo --bg") from 0.2.2 to 0.7.1: the
    /// APK build failed at `mergeDebugResources`, and nothing noticed because CI does not build
    /// Android. This reads the resource files instead of waiting for a Gradle run.
    #[test]
    fn nenhum_comentario_xml_do_android_tem_traco_duplo() {
        fn anda(d: &std::path::Path, lidos: &mut usize, achados: &mut Vec<String>) {
            for e in std::fs::read_dir(d).expect("pasta de recursos").flatten() {
                let p = e.path();
                if p.is_dir() {
                    anda(&p, lidos, achados);
                } else if p.extension().is_some_and(|x| x == "xml") {
                    *lidos += 1;
                    let texto = std::fs::read_to_string(&p).expect("xml legível");
                    for trecho in texto.split("<!--").skip(1) {
                        let comentario = trecho.split("-->").next().unwrap_or_default();
                        if comentario.contains("--") {
                            achados.push(p.display().to_string());
                        }
                    }
                }
            }
        }
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("gen/android/app/src/main/res");
        let (mut lidos, mut achados) = (0, Vec::new());
        anda(&raiz, &mut lidos, &mut achados);
        assert!(lidos > 0, "no XML read under {}: the check measured nothing", raiz.display());
        assert!(achados.is_empty(), "XML comments with a double dash (aapt2 refuses them): {achados:?}");
    }

    #[test]
    fn todo_doc_e_alcancavel() {
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let base = if raiz.join("../docs").is_dir() { raiz.join("..") } else { raiz.to_path_buf() };
        let docs = base.join("docs");
        if !docs.is_dir() {
            return; // repo sem `docs/` não tem o que medir
        }

        // Os arquivos que podem CONTER links para os docs: tudo que é índice aqui.
        let mut indice = String::new();
        for n in ["README.md", "README_br.md", "docs/README.md", "CLAUDE.md", "AGENTS.md"] {
            if let Ok(t) = std::fs::read_to_string(base.join(n)) {
                indice.push_str(&t);
            }
        }
        // E os próprios docs linkam entre si — um doc alcançado por outro doc conta.
        let mut arquivos: Vec<std::path::PathBuf> = Vec::new();
        fn anda(d: &std::path::Path, saida: &mut Vec<std::path::PathBuf>) {
            if let Ok(e) = std::fs::read_dir(d) {
                for x in e.flatten() {
                    let p = x.path();
                    if p.is_dir() {
                        anda(&p, saida);
                    } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
                        saida.push(p);
                    }
                }
            }
        }
        anda(&docs, &mut arquivos);
        for f in &arquivos {
            if let Ok(t) = std::fs::read_to_string(f) {
                indice.push_str(&t);
            }
        }

        let orfaos: Vec<String> = arquivos
            .iter()
            .filter_map(|f| {
                let rel = f.strip_prefix(&base).ok()?.to_string_lossy().to_string();
                let nome = f.file_name()?.to_string_lossy().to_string();
                // `docs/README.md` é o índice: ele não precisa ser linkado por ninguém.
                if rel == "docs/README.md" {
                    return None;
                }
                // Alcançável se alguém escreve um LINK markdown que termine no caminho ou
                // no nome do arquivo — `](docs/x.md)`, `](x.md)`, `](../docs/x.md)`.
                let alvo_a = format!("]({rel})");
                let alvo_b = format!("]({nome})");
                let alvo_c = format!("/{nome})");
                let visto = indice.contains(&alvo_a) || indice.contains(&alvo_b) || indice.contains(&alvo_c);
                (!visto).then_some(rel)
            })
            .collect();

        assert!(!arquivos.is_empty(), "docs/ vazio — a régua estaria medindo o vazio");
        assert!(
            orfaos.is_empty(),
            "documento(s) em docs/ sem NENHUM link apontando para eles:\n  {}\n\
             Um .md que ninguém alcança é trabalho que alguém vai refazer. Linke no índice \
             (README.md ou docs/README.md) ou apague.",
            orfaos.join("\n  ")
        );
    }

    /// The four iOS usage/compliance keys are in BOTH `project.yml` and the generated
    /// `Info.plist` — the regression 0.6.8 shipped and nothing measured for 26 days.
    ///
    /// `NSFaceIDUsageDescription` is the sharp one: without it iOS **terminates the
    /// process** on the first `evaluatePolicy` call, which in this app is the "Ativar
    /// Face ID" button of the first run — the first screen an Apple reviewer sees. The
    /// other three cost less but cost every time: no mic/camera description and the
    /// WebView's attachment paths die, no `ITSAppUsesNonExemptEncryption` and the export
    /// compliance question comes back on EVERY App Store Connect upload.
    ///
    /// Why it checks two files instead of one: `project.yml` is the source XcodeGen
    /// generates from, and the generated `Info.plist` is the `INFOPLIST_FILE` the target
    /// actually bundles. Keys in the yml alone are absent until somebody regenerates;
    /// keys in the plist alone die at the next `tauri ios init` — which is exactly how
    /// `aps-environment` was lost on 04/08. The pair has to hold, so the ruler measures
    /// the pair. `src-tauri/Info.ios.plist` is deliberately NOT accepted as proof: it is
    /// the file 31/07 proved `ios init` does not merge.
    #[test]
    fn as_quatro_chaves_do_ios_estao_nas_duas_fontes() {
        let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        const CHAVES: [&str; 4] = [
            "NSMicrophoneUsageDescription",
            "NSCameraUsageDescription",
            "NSFaceIDUsageDescription",
            "ITSAppUsesNonExemptEncryption",
        ];
        // gen/apple is generated and gitignored in a fresh clone of some checkouts; a
        // ruler that silently passes when the file is missing measures nothing, so an
        // absent gen/apple skips loudly rather than green.
        let fontes = [
            raiz.join("gen/apple/project.yml"),
            raiz.join("gen/apple/shvia-mobile_iOS/Info.plist"),
        ];
        if fontes.iter().any(|f| !f.exists()) {
            eprintln!("gen/apple ausente — régua pulada (rode `tauri ios init` no Mac)");
            return;
        }
        let mut faltando: Vec<String> = Vec::new();
        for f in &fontes {
            let t = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("{} ilegível: {e}", f.display()));
            for c in CHAVES {
                if !t.contains(c) {
                    faltando.push(format!("{c} em {}", f.display()));
                }
            }
        }
        assert!(
            faltando.is_empty(),
            "chave(s) de uso/conformidade do iOS ausente(s):\n  {}\n\
             Sem NSFaceIDUsageDescription o iOS ENCERRA o app no primeiro Face ID. \
             As quatro moram no bloco `info: properties:` do project.yml (fonte do \
             XcodeGen) E no Info.plist gerado (o que vai pro bundle).",
            faltando.join("\n  ")
        );
    }
}

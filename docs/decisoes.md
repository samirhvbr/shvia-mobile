# ShvIA Mobile — Decisões (ADRs)

Formato ADR. Não relitigar direção já decidida dentro de um how-to — linkar o ADR.

---

## ADR-001 — Repo separado, stack Tauri, reusando a infra de loja da Blue3

- **Data:** 07/07/2026 · **Status:** Aceito
- **Contexto:** Levar o ShvIA para **iOS + Android**. O desktop (`SHVIA-DESKTOP`)
  já é Tauri 2 (shell fino que carrega `ai.shvia.org`). A Blue3 **já publica**
  app mobile em Flutter (`BLUE3-INTRANET-MOBILE`) — então conta Apple, Play Console,
  custódia de keystore e know-how de review **já existem**.
- **Decisão:**
  1. **Repo próprio `SHVIA-MOBILE`** (não estender o `SHVIA-DESKTOP`), seguindo a
     convenção da casa (`SSHVTERM-DESKTOP` + `SSHVTERM-MOBILE`). O desktop volta a
     ser **desktop-only**.
  2. **Stack = Tauri** (não Flutter): reusa a lógica do shell do desktop (tarja
     offline, roteio de link externo), um produto coeso Rust/Tauri.
  3. **Reusar a infra de loja da Blue3** — Apple **Team ID `S65UBCTPN5`**, Google
     Play Console, custódia de keystore. Reusa **identidades/assinatura**, **não** o
     build Flutter (o build aqui é Tauri: Gradle em `gen/android` + Xcode em `gen/apple`).
  4. **Bundle ID `cloud.blue3.shvia`** — o mesmo do desktop (consistência do produto).
- **Consequências:** shell mobile-only enxuto — **sem** menu / multi-janela /
  geometria / ponte WebKitGTK / TTS-espeak. TTS = `speechSynthesis` nativo do WebView
  (iOS/Android têm voz pt-BR). **iOS exige Mac**; Android builda no Linux. Fases em
  [../.continue/escopo-mobile.md](../.continue/escopo-mobile.md).
- **Alternativa descartada:** **Flutter** — reusaria o pipeline já publicado e a
  expertise do time, mas forkaria do shell desktop e duplicaria o wrapper em Dart.

---

## ADR-002 — M3 (valor nativo p/ 4.2): sequência por tratabilidade, não por impacto

- **Data:** 16/07/2026 · **Status:** Aceito
- **Contexto:** Samir **reafirmou o Caminho B** (App Store **pública**) em 16/07,
  largando de vez a distribuição privada por **Business Manager** (as "2 semanas com
  aparelho na mesa"). Público ⇒ a regra **4.2 (web-wrapper)** se aplica ao ShvIA
  (casca fina) ⇒ **M3 é pré-requisito da submissão iOS** — no Android/Play não (o
  Play tolera wrapper). O problema: os itens do M3 têm dependências externas **muito
  diferentes**, e tratá-los na ordem errada trava tudo atrás do recurso mais caro.
- **Decisão:** sequenciar o M3 pelo que **destrava sozinho** primeiro:
  1. **Biometria (Face ID / Touch ID / BiometricPrompt)** — **100% na casca**, sem
     servidor, sem portal Apple além da assinatura normal. **FEITO PRIMEIRO** (mobile
     0.4.0). `tauri-plugin-biometric`; o gate roda na página **local** (`src/`) —
     única com bridge nativo (a página remota **não** recebe comando, ADR-001) — e
     trava o cold-start antes de navegar pro ShvIA hospedado. Espelha o modelo Blue3
     ([BIOMETRIA.md](../../../BLUE3/BLUE3-INTRANET-MOBILE/docs/MOBILE/BIOMETRIA.md)): a
     biometria é **acesso LOCAL**; o cookie de sessão same-origin continua sendo o
     auth remoto. Toggle de ativar/desativar vive na casca (não há como ser no ShvIA
     web sem furar o posture).
     **Relock on return (0.7.0, 24/09/2026, the owner's answer "right away"):** the
     gate also runs when the app comes back from the background (`WindowEvent::Resumed`:
     Android `onResume`, iOS `applicationWillEnterForeground`). The native side learns
     whether the lock is on from the local shell, through the app's only command
     (`trava_biometrica`), which the remote page still cannot reach. Only the page's
     path travels to the gate and back, never a host. A file picker opened in the last
     10 minutes spares one resume, because on Android coming back from the picker is
     itself a resume.
  2. **Push (APNs)** — **o lado SERVIDOR saiu do bloqueio: está PRONTO** desde o
     **SHVIA-WEB 2.51.0 (16/07)**, no mesmo dia deste ADR — que por isso ficou
     desatualizado até 28/07. Já existem lá: `POST /push/token` e
     `DELETE /push/token` (autenticados pela **sessão** same-origin, corpo
     `{token, platform:"ios"|"android"}`), `ApnsClient` (JWT **ES256** com a `.p8`,
     HTTP/2), `PushService::sendToUser()`, o job `SendPushNotification`,
     `php artisan push:test {user}`, `config/apns.php` e 11 testes verdes. Reuso
     confirmado: a `.p8` é **por Team `S65UBCTPN5`**, não por app → a mesma chave da
     Blue3 serve, muda só o `APNS_BUNDLE_ID` p/ `cloud.blue3.shvia`. Contrato e
     pendências em `~/x/SHVIA/SHVIA-WEB/docs/PUSH/PUSH-APNS-20260716.md`.
     **O que ainda falta é dos DOIS lados cliente:** (a) **nesta casca** —
     entitlement `aps-environment`, pedir permissão, `registerForRemoteNotifications`,
     e injetar o device token na página remota pelo mesmo caminho já usado pelo
     `window.__shviaShellVersion` (`webview.eval`), mais navegar pro `data.route` no
     tap da notificação; (b) **no front do SHVIA-WEB** — ler `window.__shviaPushToken`
     e fazer o `POST /push/token` com cookie + CSRF (e `DELETE` no logout).
     **Fora do código, do Samir:** habilitar a capability **Push Notifications** no
     App ID `cloud.blue3.shvia` (o que muda por app; a `.p8` não) e as `APNS_*` no
     `.env` de produção. **App Group não é necessário** para alerta simples — o
     requisito citado na redação original deste ADR não procede (só entraria com
     Notification Service Extension). Armadilha nº 1 registrada: build **debug**
     emite token de **sandbox** (`APNS_PRODUCTION=false`), TestFlight/loja emitem de
     **produção** (`true`). Alternativa FCM ([NOTIFICACOES.md], caminho do
     BLUE3-INTRANET-MOBILE) fica só para o **Android**, depois.
  3. **Deep-link / Universal Links** — **segue bloqueado no servidor**: exige
     `apple-app-site-association` no **SHVIA-WEB** + entitlement `associated-domains`
     (e `assetlinks.json` no Android). Conferido em 28/07: **nenhum dos dois arquivos
     existe** no SHVIA-WEB (2.88.4). Custom scheme `shvia://` é parcial e fraco
     sozinho — Universal Links é o que vale, e depende do servidor.
  4. Câmera+mic (feito), tela offline nativa (feita — reforçar), share sheet (depois).
- **Consequências:** dá pra **avançar o M3 hoje** sem depender de nada externo
  (biometria entregue na 0.4.0). **Revisão de 28/07:** o push saiu da fila do "espera
  o servidor" — o servidor chegou primeiro e o cliente é que ficou devendo, então
  **push passa a ser o próximo item do M3**, não o último. Universal Links continua
  parado no `apple-app-site-association`. Na review da Apple, citar esses recursos
  nas notas (checklist §2.1).
- **Verificação pendente:** o gate biométrico só se valida **no aparelho/simulador
  iOS** (host só faz `cargo check` + `tsc`); Face ID real precisa do Mac + device.

---

## ADR-003 — Domínio próprio `ai.shvia.org`: dual-host e fim do curinga em `is_internal`

- **Contexto/Problema:** o ShvIA saiu de `ia.blue3.com.br` para `ai.shvia.org` (a
  Blue3 ficou como financiadora, não como marca do produto). A casca mobile é
  mantida em **paridade** com a do SHVIA-DESKTOP e tinha os mesmos três pontos
  blocantes: `SHVIA_URL` (o destino), a allowlist de navegação interna e o
  `connect-src` da CSP. Nenhum deles falha de forma legível — host fora da
  allowlist faz `on_navigation` tratar a **navegação inicial** como link externo
  (o ShvIA abre no Safari e o app fica preso no splash); host fora do
  `connect-src` faz o ping de alcance ser barrado pelo WebView, e a casca fica em
  "Sem conexão" para sempre com o servidor no ar.
- **Decisão:** dual-host por allowlist **exata** de FQDN, espelhando
  `SERVER_HOSTS` do desktop: `ai.shvia.org` (canônico, o que o app abre),
  `ia.shvia.org` (CNAME do canônico) e `ia.blue3.com.br` (legado, mesmo IP, só
  durante a transição). Verificadas por DNS. Desligar o legado é remover uma
  linha.
- **Fim do curinga — mudança de COMPORTAMENTO, não só de host:** o
  `is_internal` daqui aceitava `host.ends_with(".blue3.com.br")`, isto é,
  **qualquer** subdomínio do domínio corporativo carregava dentro do app. O
  desktop fechou isso no 0.9.0 dele; o mobile herda a mesma postura agora, e
  passou a exigir **esquema** também (`https` para o servidor, `http`/`tauri`
  para a casca local). O ápex `shvia.org` fica FORA de propósito: resolve para
  outro IP e serve a landing, não o app. Coberto por 3 testes de unidade
  (`cargo test`), incluindo o sufixo-armadilha `ai.shvia.org.evil.com`.
- **Marca no splash:** o `<span>Blue3</span>` saiu e o `brand-mark.png` (a seta
  da Blue3) virou `brand-mark.svg`, a mesma marca do favicon e do badge do web. O
  splash é **pré-login**: a Blue3 só pode aparecer depois do login e só para
  e-mail dela (SHVIA-WEB/`config/brand.php`). O rodapé passou a mostrar só
  `v<versão>` — que é o que interessa saber num teste via TestFlight.
- **O `identifier` `cloud.blue3.shvia` NÃO muda.** Além de zerar dados por
  sandbox, desde a 2.51.0 do SHVIA-WEB o `APNS_BUNDLE_ID` tem de ser igual ao
  bundle id — renomear quebraria push no iOS em silêncio. E trocar bundle id de
  app publicado é app novo na loja, não atualização.
- **Consequências/limites — a ordem importa mais aqui que no desktop:** a
  atualização do mobile passa por **review da Apple**, com latência de dias. O
  build com o host novo tem de ser **submetido e distribuído ANTES** de qualquer
  redirect ou desligamento do host antigo; na ordem inversa, todo aparelho com a
  versão publicada fica sem app até o review sair. Trocar de origem também
  desloga uma vez (cookie é por origem) e zera `localStorage`/`IndexedDB` da
  WebView. Verificação: `cargo test` + `npm run build` no host; o comportamento
  de link externo (item 7 do `docs/smoke-test.md`) só se valida no aparelho.

## ADR-004 — No TLS pinning: the shell trusts the system store, on purpose

- **Context:** the shell has no certificate pinning anywhere. What exists is the exact
  host allowlist of ADR-003 (`SERVER_HOSTS` in `src-tauri/src/lib.rs`) plus the
  operating system's trust store. Measured on 21/09/2026 by content, not by file name:
  `pinn|serverTrust|didReceiveChallenge|network_security_config|TrustKit|publicKeyHash|NSPinnedDomains`
  has no hit in `src/`, `src-tauri/src/`, `plugins/` or the platform projects. Until
  now that absence had no reader: nothing said whether it was a gap or a choice.
- **What pinning would buy:** a device with a planted root CA (a corporate MDM profile,
  for example) can read and rewrite the shell's traffic, and that traffic carries the
  same-origin session cookie, which **is** the user's credential. Pinning would close
  that path.
- **Why not — the cost is specific to this product:**
  - **Rotation.** `ai.shvia.org` uses a Let's Encrypt certificate: 90 days
    (measured 24/09: issued 12/09, expires 11/12, intermediate `YE2`), renewed
    automatically, and Let's Encrypt rotates its intermediates too. A leaf or
    intermediate pin breaks on a routine renewal.
  - **The fix goes through store review.** A wrong pin is repaired only by a new build,
    and the iOS build waits days for Apple's review (see ADR-003's ordering). Until it
    ships, every installed copy is an app that cannot reach its server, and the user
    cannot tell why.
  - **A root pin shrinks the gain.** Pinning the ISRG roots would survive renewals, but
    it also breaks the day the server changes CA. It defends only against an attacker
    who can plant a CA on the device, and an MDM that can do that already manages the
    device.
  - **A WebView shell is not one switch.** The traffic is the WebView's, not a native
    HTTP client's. Whether each platform's pin mechanism covers WebView traffic has not
    been measured here. That measurement would be the first step of any revisit.
- **Decision:** no pinning. The shell keeps the exact host allowlist and the system trust
  store. Recommended in the 21/09/2026 sweep (finding `f172`) and accepted by the owner
  on 23/09/2026.
- **Revisit when:** the app starts carrying something beyond the session (tokens stored
  on the device, a second backend), a threat model with hostile device management appears,
  or the server moves to a CA and release process that can ship a pin update before a
  rotation. A revisit starts by measuring WebView coverage per platform and pins roots
  with a backup key, never the leaf.

---

## ADR-005 — One native command reaches the ShvIA page: save a file

- **Date:** 25/09/2026 · **Status:** Accepted (owner decision `mobile-download-caminho`,
  24/09/2026: *"minimal saveFile bridge on both platforms, new ADR"*).
- **Context:** the page could not download on Android. wry has no download handler there,
  and the page's save helper (`saveArtifact` in shvia-web `app.js`) falls back to a `blob:`
  link, which cannot leave the page. iOS's WebKit downloads by itself. ADR-001's posture was that
  **no** native command reaches the remote page, and the only way out for a `blob:` is a
  bridge the page can call. So this reopens ADR-001 on one point.
- **Decision:** exactly one command, `salvar_arquivo`, reachable by the ShvIA page on its
  three hosts and by nothing else:
  - **Bytes and a name in, never a path.** The page holds the authenticated session and reads
    the bytes; the shell cleans the name (last segment only, no reserved characters, never
    empty) and refuses anything over 50 MB before decoding it.
  - **The system decides where.** The official `tauri-plugin-dialog` opens "save as"
    (`ACTION_CREATE_DOCUMENT` on Android, the document picker's export on iOS), and the
    official `tauri-plugin-fs` writes Android's `content://` URI. Both are called from Rust
    only. No capability grants a `dialog:` or `fs:` permission, so no page can reach either
    plugin directly.
  - **The fence is the ACL, not a convention.** `build.rs` puts the app commands under the ACL
    (`AppManifest`). The `servidor-shvia` capability is the only one with `remote` URLs: the
    `SERVER_HOSTS`, and `allow-salvar-arquivo` and nothing else. `trava_biometrica` stays
    local-only (`default`). A test reads the capability files the build uses and fails if any
    of this drifts.
  - **The page calls it the way it already calls the desktop:** `window.__shviaCode.saveFile`,
    which `app.js` checks with `Ponte.tem('saveFile')`. The shell's injected script, which
    reaches the ShvIA hosts only, defines that one method. **Order that matters:** the web
    starts its desktop Code mode only for a bridge with `spawn` and `send` (shvia-web #360).
    That has to be in production before this ships, because the object's mere presence used
    to boot Code mode and throw at load.
- **Consequences:** the phone saves where the person chooses, on both platforms, with the
  same web code as the desktop. iOS keeps a staged copy in the app's Documents only for the
  length of the dialog (the plugin exports a file that already exists there; ours is removed
  after).
- **Alternatives set aside:**
  - *A share sheet.* It needs native code of our own on both platforms (FileProvider + intent;
    `UIActivityViewController`). The official dialog covers "put this file somewhere" without
    any.
  - *Writing to app-specific storage.* On Android 11+ the Files app cannot reach
    `Android/data`, so the file would exist and nobody could find it.
  - *A generic file bridge* (paths, reads). That is exactly the surface ADR-001 kept closed.
- **Revisit when:** a second native command is proposed for the page, or the page needs to
  READ a file (that is a different fence and a different ADR).

[BLUE3-MOBILE-SERVICOS-AO-VIVO.md]: ../../../BLUE3/BLUE3-INTRANET-MOBILE/docs/BLUE3-MOBILE-SERVICOS-AO-VIVO.md
[NOTIFICACOES.md]: ../../../BLUE3/BLUE3-INTRANET-MOBILE/docs/NOTIFICACOES.md

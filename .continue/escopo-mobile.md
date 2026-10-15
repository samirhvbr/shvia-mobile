# ShvIA Mobile — Roadmap de Produção (M0–M5)

> **Nota de revisão documental — 15/10/2026.** Este roteiro mantém propostas e
> medições datadas. O retrato atual da implementação fica em
> [docs/current-state.md](../docs/current-state.md); operação em
> [docs/operations.md](../docs/operations.md). Caixas antigas não confirmam o estado
> atual das lojas/aparelhos. Nenhum item pendente foi removido por esta revisão.

> Este repo é o cliente **mobile** (iOS + Android) do ShvIA, em **Tauri 2** (ADR-001).
> Irmãos: `SHVIA-DESKTOP` (desktop) e `SHVIA` (servidor Laravel). WIP mora aqui;
> quando amadurece, migra para `docs/`.
>
> **Decisões travadas (07/07/2026):** stack **Tauri**; repo separado; publicar nas
> **lojas** (App Store + Play) reusando a infra da Blue3; bundle **`cloud.blue3.shvia`**.

---

## 1. Premissa

**Shell fino Tauri 2** carregando `https://ai.shvia.org` no WebView nativo
(iOS = WKWebView, Android = System WebView). Servidor = fonte da verdade. Não é
reescrita — é o mesmo conceito do desktop, mobile-only.

**Auth (comportamento aprovado — Samir, 07/07/2026):** cookie de sessão
same-origin; sessão válida → **entra direto, sem pedir credencial** (validado no
desktop Linux). Sem autofill de usuário/senha no cliente (segredo zero no shell,
ADR-001). No iOS, conferir a persistência real do cookie entre reinícios no
smoke-test (item 2 de [../docs/smoke-test.md](../docs/smoke-test.md)).

## 2. Barreiras (e o que já está resolvido)

1. **iOS só builda em macOS + Xcode.** Android builda no Linux. (Há Mac/CI macOS.)
2. **App Store 4.2 (web wrapper).** WebView que só abre um site pode ser reprovado →
   **M3 (valor nativo) é pré-requisito de submissão iOS.** Play é mais tolerante.
3. **Contas/assinatura/loja — JÁ EXISTEM.** A Blue3 publica `BLUE3-INTRANET-MOBILE`
   (Flutter): Apple **Team ID `S65UBCTPN5`**, Play Console, custódia de keystore.
   Reusar (ver §3). Deixa de ser barreira.
4. **UX mobile = responsividade do ShvIA web.** Como é shell fino, o celular mostra
   o Blade remoto → trabalho no repo **`SHVIA`** (Laravel), não neste (M5).

## 3. Reuso da infra Blue3 (não é greenfield)

- **Apple:** mesma conta/App Store Connect; Team ID `S65UBCTPN5`, signing
  `automatic`, method `app-store-connect` → `tauri.ios.conf.json`
  (`bundle.iOS.developmentTeam`).
- **Google Play:** mesma org; keystore via `key.properties` (não versionado) + Play
  App Signing → replicar no `gen/android`.
- **Não** transfere o *build* Flutter (Runner/Xcode + `flutter build`) ≠ Tauri
  (Gradle em `gen/android` + Xcode em `gen/apple`). Reusa identidades, não pipeline.
- **Build number** monotônico (iOS `CFBundleVersion` / Android `versionCode`):
  padrão deles `X.Y.Z+<AAAAMMDDHHMM>`; adotar equivalente a partir do `version.md`.

## 4. Fases

### M0 — Scaffold do shell mobile-only ✅ (feito)
Repo criado espelhando o desktop, mas mobile-only: `src-tauri` enxuto (só tarja
offline + roteio de link externo; sem menu/multi-janela/WebKitGTK/TTS-espeak),
`cargo check` no host **passa**. Targets Rust Android instalados no host.

### M1 — Android (no Linux) — **build OK ✅**, smoke-test on-device pendente
- ✅ Toolchain **reusado da máquina** (dev Flutter da Blue3), zero download: JDK 17
  (`~/Android/jdk-17`), SDK (`~/Android/Sdk`, platforms 34-36, build-tools, adb),
  **NDK 28.2**. Requer `JAVA_HOME`/`ANDROID_HOME`/`NDK_HOME` no ambiente (documentar
  em `docs/build.md` ou num `build-local` de Android).
- ✅ `tauri android init` → `gen/android` (versionado); `tauri android build --debug`
  produz **APK + AAB** (arm64), Rust compilado via NDK. Cadeia validada de ponta a ponta.
  `applicationId = cloud.blue3.shvia`.
- ⏳ **SMOKE-TEST #1 (crítico):** `/chat` **streaming (SSE)** na WebView Android + login
  → cookie persistente + mic — **precisa de aparelho/emulador** (esta máquina não tem
  KVM/device). `tauri android dev` com um device via adb, ou emulador com KVM.
- ⏳ Keystore (reuso do padrão Blue3: `key.properties` + Play App Signing), **AAB
  assinado** (release), script `build-local` de Android.

### M2 — iOS (no Mac) — **em andamento (foco atual, 14/07)**
- ✅ **Pré-config feita do Linux (0.2.6):** `bundle.iOS.developmentTeam =
  S65UBCTPN5` no `tauri.conf.json` (o init já sai assinável) +
  `src-tauri/Info.ios.plist` com mic/câmera (o Tauri mescla no Info.plist
  gerado) + ícones iOS prontos desde 0.2.2.
- **MacBook disponível (07/07)** — o desktop 0.5.5 rodou lá com `npm run tauri dev`.
- ✅ **Lado Mac feito (0.3.0, 14/07):** `tauri ios init` → `gen/apple` gerado
  (Team + bundle conferidos no `project.pbxproj`; mic/câmera mesclados no
  Info.plist) + `tauri icon` (Xcode assets) + `tauri ios dev` → **app abriu
  logado no simulador iPhone 17 Pro Max**. Atenção: `tauri icon` reseta o
  `ic_launcher_background.xml` do Android p/ `#fff` — restaurar o navy `#0D1B2A`
  após rodar (feito em 0.3.0).
- ✅ **Smoke-test parcial no simulador (0.3.1, 14/07):**
  - **Item 1 ✓** — offline nativo: relaunch mostrou "Sem conexão — reconectando…"
    e **entrou sozinho** sem clique (auto-retry ok).
  - **Item 2 ✓** — cookie **persistiu** após `simctl terminate` + relaunch (a
    dúvida do WKWebView está respondida: entra logado direto).
  - **Item 3 ✓ (0.3.4, re-teste pós-2.17.11)** — **streaming token a token
    confirmado visualmente**: 3 frames seguidos com o raciocínio crescendo
    palavra a palavra (cursor de digitação visível) e timer ao vivo; resposta
    final 1.239 tok @ 22 tok/s. Enter do teclado físico **envia** (form submit
    funciona mesmo com o quirk de digitação do simulador).
  - **Item 4 ~** — botão "ouvir" presente; áudio precisa de ouvido humano.
  - **Itens 5–7** (mic, anexos, links externos) pendentes — precisam de humano.
  - **Item 8 ✓ (re-teste)** — teclado virtual sobe e o composer levanta junto,
    campo focado visível, sem cobertura; digitação pelas teclas na tela funciona.
  - **Item 9 ~ (re-teste)** — portrait ok (rotação DURANTE streaming não quebrou
    a geração); landscape segue layout desktop (sobra conhecida da 2.17.11).
- 🐞 **BUG BLOQUEADOR — CORRIGIDO no SHVIA-WEB 2.17.11 (14/07), aguardando
  deploy.** Causa-raiz achada com o CSS de produção (md5 == repo): em ≤768 px o
  painel do meio vira drawer `position:fixed` **default-aberto** com fundo
  `--bg-panel` = rgba 2,5% → um **vidro invisível por cima do chat**. O
  "hit-testing deslocado ~400 px" era isso: o chat aparecia ATRAVÉS do drawer,
  mas os toques acertavam o drawer. Fix (validado em harness 440×956 com o CSS
  real): drawer inicia FECHADO no mobile, fundo sólido `#11161e`, backdrop
  toque-fora-fecha, chip "Projeto" da topbar abre/fecha (o stub de reabrir é
  display:none no mobile — não havia porta), safe-area na topbar e no drawer
  (a regra mobile antiga da topbar era letra morta: a base "pit-wall" vinha
  depois no arquivo e vencia a cascata). Cadastro: iOS dava **zoom automático**
  ao focar campo com fonte <16 px (o "corte lateral") → inputs a 16 px +
  `viewport-fit=cover` + safe-area no layout auth. **Sobra conhecida:**
  landscape (item 9) segue layout desktop (956 px de largura > breakpoint) —
  avaliar `max-height` como critério de mobile depois do re-teste.
- ✅ **2.17.11 deployado (14/07 ~11h)** — confirmado via `/api/v1/health`
  (`version.app`). Chat em coluna única no iPhone, toques funcionando (Samir
  digitou e conversou no simulador). Sobra nova: **auto-zoom do iOS ao focar o
  composer do chat** (o fix de fonte 16 px da 2.17.11 cobriu o auth, não o
  composer) — mesma correção, outro campo.
- 🔧 **Falso "Sem conexão" resolvido na casca (0.3.4):** o ping de saúde saiu de
  `/api/v1/health` (sonda 14 provedores em série; em cache frio >6 s = estourava
  o timeout da casca) para o **`/up`** nativo do Laravel (~30 ms). Diagnóstico
  completo no commit. Pendências do LADO SERVIDOR reveladas pelo health:
  `redis: status error`, `gemini`/`perplexity` com `http_404` na sonda, e
  (opcional) desacoplar a sonda pesada do request (cache maior/refresh em
  background) para o health não depender do humor dos provedores.
- Quirk de teste (não é bug do app): teclado de hardware do simulador não digita
  no campo do chat (WKWebView); **colar** (long-press → Paste) funciona. Testar
  teclado virtual em aparelho físico.
- Depois do smoke-test: primeiro build no **TestFlight**.

### M3 — Valor nativo (destrava App Store 4.2) — **OBRIGATÓRIO (decisão 15/07)**
Push (APNs), biometria (Face ID), compartilhar, deep-link `shvia://`, tela offline
nativa, safe-area/status bar/splash. **Samir escolheu o Caminho B — App Store PÚBLICA,
mesmo trilho do `~/x/BLUE3/BLUE3-INTRANET-MOBILE`** (`app-store-connect`, conta S65UBCTPN5,
custo zero novo). Como o BLUE3 é Flutter NATIVO ele passou a 4.2 fácil; o ShvIA é
**shell fino (wrapper) → a 4.2 se aplica**, então o M3 deixa de ser opcional. Push =
reusar a infra APNs da Blue3 (.p8/App Group já existem). Ver checklist §2.1.

**Progresso (ADR-002 — sequência por tratabilidade, não por impacto):**
- [x] **Biometria (Face ID / Touch ID) — mobile 0.4.0 (16/07).** `tauri-plugin-biometric`
  como plugin **mobile-only** (`[target.'cfg(ios/android)']` + registro atrás de
  `#[cfg(mobile)]`; capability `mobile.json` escopada a iOS/Android — declarar no
  `default` quebraria o build host). Gate na casca **LOCAL** (`src/main.ts`, a única
  com bridge nativo): opt-in na 1ª execução, desbloqueio no cold-start antes de
  navegar pro remoto, "Desativar bloqueio" exige re-autenticar; `NSFaceIDUsageDescription`
  no `Info.ios.plist`; passcode do aparelho como fallback. **Não** substitui o cookie
  de sessão (é acesso LOCAL, modelo Blue3). **Verificado no host:** `cargo check`
  (host **e** `aarch64-apple-ios`) + `tsc`/`vite build` passam. **Pendente:** Face ID
  real no simulador/aparelho (novo item do smoke-test — só se valida com device).
- [~] **Push (APNs) — CÓDIGO COMPLETO nas duas pontas (01/08, mobile 0.6.0 +
  web 2.91.7); falta portal/env/smoke.** Servidor pronto desde a 2.51.0
  (16/07). O que entrou em 01/08:
  - **Casca:** plugin interno **`plugins/tauri-plugin-shvia-push`** (padrão do
    biometric; iOS-only, registrado atrás de `#[cfg(target_os = "ios")]`).
    O app do Tauri NÃO tem AppDelegate Swift (o delegate é classe ObjC do tao)
    → o plugin ADICIONA os seletores `didRegisterForRemoteNotifications...`/
    `didFail...` via `class_addMethod` no `load()` (pesquisa 01/08: é o padrão
    dos plugins de comunidade e do PR draft tauri#11652; não há plugin oficial).
    Token sobe por `Channel` → Rust guarda em estado + injeta
    `window.__shviaPushToken` (+ `__shviaPushPlatform` + evento
    `shvia:push-token`) na página remota a cada page load; tap em notificação
    entrega `userInfo.route` → navegação SANITIZADA (só caminho relativo).
    Permissão pedida no 1º load REMOTO (usuário logado, com contexto), em
    thread própria (o comando bloqueia até o usuário decidir). Página remota
    segue sem NENHUM comando nativo (ADR-001). Entitlement `aps-environment`
    = `development` no `.entitlements` (o export p/ ASC troca p/ production).
    `cargo check` host + `aarch64-apple-ios` ✓.
  - **Front (SHVIA-WEB 2.91.7):** `registerPushToken`/`unregisterPushToken`
    no app.js — boot + evento, POST sessão+CSRF, DELETE no logout.
  **Restam do Samir, fora do código:** capability **Push Notifications** no
  App ID `cloud.blue3.shvia` (portal) e `APNS_*` no `.env` de produção
  (`APNS_PRODUCTION=true` p/ TestFlight/loja!). Depois: smoke on-device
  (`php artisan push:test` → notificação no aparelho → tap navega).
  Nota: App Group não é preciso para alerta simples.
- [ ] **Universal Links** — segue bloqueado: conferido em 28/07, **nem
  `apple-app-site-association` nem `assetlinks.json` existem no SHVIA-WEB 2.88.4**;
  falta também o entitlement `associated-domains`.
- [~] Câmera+mic (feito) · tela offline nativa (feita).

### M4 — Publicação nas lojas — **FASE ATIVA (30/07)** → runbook em [../docs/testflight-checklist.md](../docs/testflight-checklist.md) §0
Doc criado (15/07) em 2 partes: **Parte 1 = TestFlight interno** (fecha o M2, sem
4.2) e **Parte 2 = App Store pública** (4.2, nutrition label, screenshots). Já
adiantado do Linux (mobile 0.3.11): `PrivacyInfo.xcprivacy` (sem tracking,
UserDefaults CA92.1), `ITSAppUsesNonExemptEncryption=false`, `minimumSystemVersion
14.0` + `category productivity`.

**Reordenado em 30/07 com o alvo "publicar amanhã".** O que trava deixou de ser o
build e passou a ser a papelada e o `gen/apple` defasado:
1. 🔴 **URL de política de privacidade + suporte** — não existem; sem a primeira o
   App Store Connect **não deixa submeter**. Repo `SHVIA-SITE`.
2. 🔴 **`tauri ios init` no Mac** — conserta de uma vez o `Info.plist` sem
   `NSFaceIDUsageDescription` (crash na 1ª tela), o `PrivacyInfo.xcprivacy` fora
   do alvo e a versão `0.2.6`.
3. 🟡 **Smoke-test on-device do item 10 (biometria)** — caminho nunca exercido em iOS.
4. Build/upload + itens `[ASC]` (ficha, screenshots, nutrition label, idade).

### M5 — Responsividade mobile do ShvIA web (repo `SHVIA`)
`viewport`, `env(safe-area-inset-*)`, teclado, nav/sidebar colapsável, alvos de toque.
Pré-requisito de qualidade p/ submeter.

## 5. Estado

### 🔴 Retrato de 29/08/2026 — o reenvio está parado há 17 dias, e é o único item

**Onde está:** a submissão foi **rejeitada em 12/08** por dois motivos
(2.1(a) microfone morto e 5.1.1(v) exclusão de conta inalcançável). **As duas
causas estão CORRIGIDAS E NO AR** — conferido em 29/08 por md5: o
`ai.shvia.org/js/app.js` de produção é byte-a-byte igual ao do master local
(web 2.110.9x), com o guarda `iOSWebView` do microfone e o
`DELETE /api/v1/me/account` da exclusão. As páginas `shvia.org/privacidade.html`
e `/suporte.html` respondem 200 com CNPJ preenchido. O binário **0.6.5 segue
válido no ASC**: o reenvio é resposta + submit, **não precisa de build novo**.

**⚠️ Achado de 29/08 que muda a instrução do vídeo: a exclusão de conta mudou de
endereço de novo.** O retrato de 22/08 mandava gravar "o fim da aba Perfil"; o
redesign A6 (web 22/08) tirou de lá e deu painel próprio. Terceiro endereço em
duas semanas — gravar o caminho errado é queimar mais um ciclo de review.
**Caminho de hoje:** Configurações → grupo *Sistema* → **Conta & zona de risco**
→ *Quero excluir minha conta* → senha → *Excluir definitivamente*
(`#pane-conta` no `dashboard.blade.php` do SHVIA-WEB).

**O que falta é tudo do lado do Samir** (ordem em
[`docs/testflight-checklist.md` §3](../docs/testflight-checklist.md), que agora
traz os textos do Resolution Center e das Notes prontos para colar):

1. ~~testar no aparelho~~ — **feito em 29/08**: microfone sumiu do composer e a
   exclusão apagou a conta de verdade; o iPhone tem o **0.6.5**, o build em review;
2. **gravar o vídeo** do fluxo de exclusão em aparelho físico, no caminho ATUAL.
   ⚠️ Conferido em 29/08: o "a Apple exige o vídeo" era **paráfrase nossa** repetida
   em quatro arquivos — a mensagem literal da rejeição não está guardada em lugar
   nenhum. O exigido é indicar o caminho; o vídeo é reforço barato. Guardar o texto
   original do Resolution Center em `docs/` na próxima;
3. responder no **Resolution Center** apontando as duas correções (texto pronto);
4. ~~conferir a conta demo~~ — **feita em 29/08:** é `apple@shvia.org`
   ("Apple Corp", 05/08/2026), viva e com a senha do ASC. O endereço
   `apple-review@shvia.org` que a doc dizia **nunca existiu**;
5. **Submit for Review** com o MESMO build 0.6.5.

**Não fazer:** subir build novo "para garantir". O 0.6.5 é iPhone-only e é o que
foi revisado; divergir do que está publicado custa um ciclo inteiro de review.

### 🚀 Revisão de 30/07 — véspera da publicação

Revisão pedida pelo Samir com o alvo "publicar amanhã". **Verde no host** (rodado
hoje, 30/07): `cargo check` ✓ · `cargo test` **3/3** ✓ · `tsc` + `vite build` ✓ ·
`npm audit` **0 vulnerabilidades** ✓ · `ai.shvia.org/up` **200 em 32 ms** ✓. O
`version.md` estava **0.5.2** e o resto da árvore em **0.5.1** — o `sync-version`
só roda no `prebuild`, então quem não buildou não propagou; reconciliado em 0.5.3.

**Achados novos, do mais grave pro menos:**

1. 🔴 **`gen/apple` defasado desde 14/07 — três problemas no MESMO arquivo.** O
   `INFOPLIST_FILE` do alvo Xcode é `gen/apple/shvia-mobile_iOS/Info.plist`
   (conferido no `project.pbxproj`), e ele foi gerado na 0.3.0 a partir do
   `Info.ios.plist` **daquela** data. Tudo que o `Info.ios.plist` ganhou depois
   nunca chegou lá:
   - **`NSFaceIDUsageDescription` ausente** (entrou na 0.4.0). Sem essa chave o
     iOS **encerra o processo** na 1ª chamada de `evaluatePolicy` — que neste app
     é o botão "Ativar Face ID" do card de 1ª execução, **a primeira tela que um
     revisor da Apple vê**. Como o único build que rodou em aparelho foi o
     0.3.13/0.3.14 (**anterior** à biometria), esse caminho nunca foi exercido em
     iOS de verdade. Era o risco nº 1 da submissão.
   - **`ITSAppUsesNonExemptEncryption` ausente** (entrou na 0.3.11) → a pergunta
     de conformidade de exportação volta a cada upload.
   - **Versão literal `0.2.6`** no `Info.plist` **e** no `project.yml`, com o app
     em 0.5.x (o gotcha do checklist §1.3, agora com causa entendida: não é o
     build que "esquece", é a cópia gerada que nunca foi refeita).

   Corrigido à mão na **0.5.3** (as três chaves + versão), mas a cura de raiz é
   **rodar `tauri ios init` de novo no Mac**, que regenera o `gen/apple` inteiro
   a partir das fontes atuais.
2. 🔴 **`PrivacyInfo.xcprivacy` NÃO está no alvo Xcode.** O arquivo existe em
   `gen/apple/shvia-mobile_iOS/` desde a 0.3.11, mas **`grep PrivacyInfo
   project.pbxproj` não acha nada** — o `.xcodeproj` é de 14/07, anterior ao
   arquivo. Ou seja: o privacy manifest **não entra no bundle**. Isso confirma o
   item aberto do checklist §1.2 e é outra coisa que o `ios init` conserta (o
   `project.yml` já lista `shvia-mobile_iOS` como source; falta o XcodeGen rodar).
   Não dá pra consertar com segurança do Linux — editar `project.pbxproj` à mão
   arrisca quebrar o build inteiro por ganho zero, já que o `init` refaz.
3. 🟡→🟢 **Sem URL de política de privacidade — bloqueio DURO do App Store
   Connect. RESOLVIDO NO MESMO DIA.** Não existia página de privacidade nem de
   suporte em lugar nenhum. Escritas em 30/07 no `SHVIA-SITE` **0.4.0**
   (`privacidade.html` + `suporte.html`), fundamentadas no código real do
   SHVIA-WEB — colunas de `users`, BYOK por usuário, `data_locality`
   (on-prem/EUA/China), zeragem do RAG fora do on-prem, `maskOutboundPii`,
   toggle `lgpd_strict`, retenção de 180 dias agendada em `routes/console.php`.
   A tensão com o ADR-001 do site ("nada de Blue3 na landing") virou o **ADR-014**
   de lá: a exceção vale só para o parágrafo de identificação legal.
   **Sobra do Samir:** razão social + CNPJ (a página tem `[PREENCHER]` e o
   harness trava o deploy até preencher), criar `privacidade@`/`suporte@shvia.org`
   e publicar. Ver o checklist §0.
4. 🟡 **Push segue 0% no cliente.** Reconferido hoje: o servidor está pronto
   (`POST`/`DELETE /push/token` no `routes/web.php` do SHVIA-WEB, `config/apns.php`,
   `APNS_*` no `.env.example` já com `TEAM_ID=S65UBCTPN5` e
   `BUNDLE_ID=cloud.blue3.shvia`), mas **`grep __shviaPushToken` no
   `public/js/app.js` não acha nada** e a casca não tem entitlement nem registro.
   É o gancho mais forte contra a 4.2 e continua sendo a maior aposta em aberto.
5. 🟢 **Universal Links: reconfirmado impossível hoje.** Nem
   `public/.well-known/apple-app-site-association` nem `assetlinks.json` no
   SHVIA-WEB 2.88.8. Sai da conversa de amanhã.
6. 🟢 **Ícones OK.** Os 18 do `AppIcon.appiconset` estão sem canal alpha,
   incluindo o `AppIcon-512@2x.png` (1024×1024 RGB) — o gotcha da 0.3.14 está
   resolvido e não volta.
7. 🟢 **Android `tauri.properties` se autocura.** Está em `versionName 0.2.2 /
   versionCode 2002`, mas o próprio arquivo se declara autogerado e o Tauri o
   reescreve a cada build a partir do `tauri.conf.json` — ao contrário do iOS,
   aqui a defasagem é cosmética.
8. 🟢 **Lacuna conhecida do Android (não bloqueia iOS):** o `AndroidManifest.xml`
   só declara `INTERNET` — falta `RECORD_AUDIO` e `CAMERA` para o ditado e o
   anexo por foto funcionarem na WebView. Some junto com o smoke-test on-device
   que já estava pendente.

### ✅ 31/07 — dia da publicação: lado máquina CONCLUÍDO (0.5.4)

Executado no Mac, na ordem do runbook §0 do checklist:

1. **`tauri ios init` rodado** (xcodegen 2.46): `PrivacyInfo.xcprivacy` entrou no
   alvo e no IPA. **Descoberta:** o init **não mescla** o `Info.ios.plist` no
   Info.plist gerado (removeu as 4 chaves; restauradas do 0.5.3 — regra nova no
   checklist). `tauri icon` não foi rodado (init não tocou ícones; evita o
   gotcha do alpha/navy).
2. **Gotcha novo:** o cache de `target/` com caminhos pré-mudança de pasta
   (`~/x/SHVIA-MOBILE`) quebra também o build release iOS — mesma cura
   (rm build/.fingerprint), agora documentada no checklist.
3. **`ShvIA.ipa` de distribuição gerado e auditado** (export app-store-connect;
   PrivacyInfo/FaceID/encryption/versão conferidos por unzip no payload).
4. **Bloqueio duro do ASC destravado:** `privacidade.html` 0.4.1 com controlador
   preenchido (BLUE3 TECNOLOGIA LTDA, CNPJ 19.648.136/0001-30) — commitado e
   pushed no SHVIA-SITE; harness verde.

**Resta (humano):** deploy do site no servidor + caixas `privacidade@`/`suporte@`
+ conferência do scheduler de retenção · smoke on-device com item 10 (biometria)
primeiro · upload do IPA (Organizer/Transporter — sem chave ASC de API no Mac) ·
ficha da loja no ASC · decisão "submeter já × esperar o push".

**Smoke no simulador (31/07, iPhone 17 Pro):** app lança sem crash ✓ · login
remoto renderiza em coluna única ✓ · caminho "sem biometria disponível" degrada
graciosamente (main.ts:160 pula o gate — simulador sem Face ID enrolled) ✓ ·
caminho COM Face ID pendente (exige Features ▸ Face ID ▸ Enrolled no menu do
Simulator — CLI não controla; ou o aparelho físico). Achado de M5 no smoke: o
botão "Exportar conversa" da topbar poluía o mobile → escondido no gatilho
mobile em **SHVIA-WEB 2.89.5** (chat-workspace.css, pedido do Samir).

### 🔎 Revisão de 28/07 — o que o mobile deixou passar enquanto o resto andava

O roadmap abaixo estava congelado em **16/07**. Nesse intervalo o **SHVIA-WEB foi
de 2.19.2 a 2.88.4** (~70 versões) e o desktop, de 0.9.x a **1.1.9**. O que a
revisão achou, do mais caro pro mais barato:

1. **Push: o servidor chegou primeiro e ninguém avisou o mobile.** Pronto desde a
   **2.51.0 (16/07)**. O roadmap seguia dizendo "bloqueado no SHVIA-WEB" por 12
   dias. Como push é **o gancho mais forte da regra 4.2**, isso é o caminho
   crítico da loja pública parado por desinformação, não por falta de recurso.
   Detalhes na fase M3.
2. **M5 marcado `[x]` com base na 2.19.2 — e o web mudou muito desde então.**
   Entraram, entre outras, o **Modo Code** (topbar com toggle de motor, timeline
   por projeto), a **memória (ai-memory)** e os overlays de saída. Nada disso
   passou por um olho mobile. **O `[x]` do M5 vale para o que existia em 15/07**;
   as telas novas são território não verificado — reteste antes de mandar
   screenshot pra loja.
3. **Ambiente local quebrado (consertado nesta revisão, 0.5.1).** O repo mudou de
   `~/x/SHVIA-MOBILE` para `~/x/SHVIA/SHVIA-MOBILE` e o `src-tauri/target` (3,9 GB)
   guardava caminhos absolutos antigos → `cargo check` morria em
   `failed to read plugin permissions ... No such file or directory`. Sintoma
   confuso: parece erro de plugin, é cache. Cura: apagar `target/debug/build` e
   `target/debug/.fingerprint`. O `node_modules` também era de 07/07 e **não tinha
   o `@tauri-apps/plugin-biometric`** (entrou na 0.4.0, 16/07) — ou seja, ninguém
   buildava a casca aqui desde antes da biometria.
4. **Toda a doc apontava para os caminhos velhos dos repos irmãos** (`~/x/SHVIA-WEB`,
   `~/x/SHVIA-DESKTOP`, `~/x/BLUE3-INTRANET-MOBILE`) — corrigido para
   `~/x/SHVIA/…` e `~/x/BLUE3/…`, inclusive os links relativos do ADR-002.
5. **`gen/apple/Info.plist` ainda congelado em `0.2.6`** e o `tauri.properties` do
   Android em `versionName 0.2.2 / versionCode 2002`, com o app em **0.5.0**. Já
   estava anotado como gotcha no checklist §1.3 e continua valendo — conferir no
   Organizer antes do upload.
6. **Verificação verde no host, pós-conserto:** `cargo check` ✓ · `cargo test`
   **3/3** ✓ (allowlist de host) · `tsc` + `vite build` ✓ · `npm audit` **0
   vulnerabilidades** (era 1 alta, postcss 8.5.16 → 8.5.24).

### Fases

- [x] M0 · [~] M1 (build OK; falta smoke-test on-device) · [x] **M2 FECHADO 🎉**
  (15/07: app instalado e RODANDO no iPhone físico via TestFlight interno, build
  0.3.13/casca 0.3.14; smoke 1–3, 8, 9-portrait ✓ no simulador; itens 4–7
  TTS/mic/anexos/links agora dá pra fechar NO APARELHO REAL) · [ ] M3 · [ ] M4 ·
  [x] M5 (2.17.11 drawer-vidro/safe-area ✓ · 2.17.12 auto-zoom do composer ✓
  e topbar 2 linhas + versão visível · 2.17.13 fonte por usuário ·
  **2.17.16 clip lateral ✓** — era min-width:auto nos itens da grade
  .chat-area, #messages esticava a 687px/440; diagnóstico por badge injetado
  pela casca no WKWebView · **2.19.2 landscape ✓** — ver abaixo)
- **DECISÃO landscape (15/07): é BUG p/ telefone, FEATURE p/ tablet.** O gatilho
  mobile era só `max-width` (768px); deitado o iPhone tem ~932px de largura e
  caía nas 3 colunas de desktop (sidebar 320px comendo a tela, thread com ~268px
  de altura útil). Fix (SHVIA-WEB **2.19.2**, validado em harness 932×430 com o
  CSS/JS reais): adicionado `(max-height: 480px)` como 2º gatilho de mobile aos
  17 blocos `@media` (8×768 + 9×720) e aos 3 `matchMedia` do app.js. 480px
  separa telefone-deitado (~430px alto) de tablet-deitado (iPad landscape
  744–834px) — **iPad em paisagem segue em colunas de propósito**. Não-regressão
  confirmada: portrait (390×844) e desktop (1280×800) idênticos. **2.19.2
  DEPLOYADA (15/07)** — confirmado: `/api/v1/health` version.app=2.19.2 e os
  assets em produção têm o gatilho (CSS 17× / JS 3× `max-height: 480px`).
- **Próximo (revisto em 30/07 — modo publicação):** o caminho crítico deixou de
  ser "que recurso construir" e passou a ser **"o que impede o upload de sair"**.
  Ordem de amanhã no
  [../docs/testflight-checklist.md](../docs/testflight-checklist.md) §0, resumida:
  1. **Página de privacidade + suporte no `shvia.org`** — bloqueio duro do App
     Store Connect, e o mais barato de tirar do caminho.
  2. **`tauri ios init` no Mac** — refaz o `gen/apple` e mata de uma vez o
     `Info.plist` defasado (Face ID!), o `PrivacyInfo.xcprivacy` fora do alvo e a
     versão 0.2.6.
  3. **Smoke-test no iPhone físico, com o item 10 (biometria) primeiro** — é o
     caminho que nunca rodou em iOS e o que um revisor vê primeiro.
  4. **Build + upload + TestFlight interno** (sem beta review) → só então decidir
     a submissão pública.
  **Push continua sendo o item de maior alavancagem do produto**, mas não cabe
  entre hoje e amanhã: exige Swift no `gen/apple`, entitlement, capability no
  portal, `APNS_*` em produção e o `POST /push/token` no front do SHVIA-WEB — e
  nada disso se testa fora do aparelho. Ele é a decisão "submeter já e arriscar a
  4.2" × "segurar uma semana e submeter com push".
  **Gotchas da 1ª submissão iOS documentados** em
  [../docs/testflight-checklist.md](../docs/testflight-checklist.md): buildar SÓ
  pelo terminal (Xcode GUI = `npm: command not found`), ícone 1024 sem canal alpha
  (`magick -alpha remove`), upload via Transporter, teste interno não usa código.

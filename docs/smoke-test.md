# Smoke-test móvel (Android + iOS) — roteiro pro MacBook

> Valida o shell em aparelho/simulador de verdade. O item crítico é o **/chat
> streaming (SSE)** — o resto é confirmação. Android também roda no Linux (com
> device USB); iOS **só** no macOS.

---

## Checklist (as duas plataformas)

| # | Teste | Esperado |
|---|-------|----------|
| 1 | **Splash** ao abrir | Marca com sonar → entra no ShvIA. Sem rede: "Sem conexão com o servidor — reconectando…" + **Tentar agora** (sem tarja vermelha na casca; a tarja continua nas páginas remotas). **Religar a rede → entra sozinho** em ≤5 s, sem clique |
| 2 | **Login + persistência** | Logar; fechar o app; reabrir → **entra direto** sem pedir credencial (cookie de sessão persistiu — comportamento aprovado 07/07). No iOS conferir de verdade: o WKWebView às vezes atrasa o flush de cookies |
| 3 | **/chat SSE** ⭐ | Perguntar à Anna → resposta **streama token a token** (não chega tudo de uma vez no fim). É o smoke-test #1 do projeto |
| 4 | **TTS "ouvir"** | Play na resposta → fala **pt-BR** (speechSynthesis nativo: iOS tem voz pt-BR; Android depende do engine TTS do aparelho) |
| 5 | **Mic/ditado** | Se o botão aparecer: permissão de mic + transcrição funcionando |
| 6 | **Anexos** | 📎 abre o seletor nativo; imagem sobe e aparece no chat |
| 7 | **Links externos** | Link fora dos hosts do servidor (`ai.shvia.org`, `ia.shvia.org`, `ia.blue3.com.br` — ver `SERVER_HOSTS` em `src-tauri/src/lib.rs`) abre no **navegador do sistema**, não dentro do app. Inclui o ápex `shvia.org`, que é a landing e NÃO é o app |
| 8 | **Teclado virtual** | Compositor não fica coberto pelo teclado; safe-area ok (notch/gesto) |
| 9 | **Rotação** | Paisagem não quebra o layout (a topbar rola na horizontal) |
| 10 | **Biometria (Face ID/Touch ID)** ⭐M3 | **1ª execução:** card "Proteger o ShvIA com Face ID?" → **Ativar** dispara o prompt do SO → sucesso entra no ShvIA. **Reabrir (cold-start):** prompt do Face ID **auto-abre**; sucesso → entra; falha → "Desbloquear" tenta de novo e aparece "Desativar bloqueio" (que **re-pede** biometria). **Agora não** na 1ª vez → nunca mais pergunta. No simulador: enrolar Face (Features ▸ Face ID ▸ Enrolled) e usar Matching/Non-matching Face. Sem biometria no aparelho → entra direto (sem trava) |

Anotar resultado por plataforma no [.continue/escopo-mobile.md](../.continue/escopo-mobile.md) §Estado.

---

## Android (MacBook ou Linux + aparelho)

**Caminho rápido — instalar o APK debug pronto** (buildado no Linux):

```bash
# no aparelho: ativar Depuração USB (Config > Sobre > 7 toques em "Número da versão")
adb devices                      # aparelho tem que listar como "device"
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

**Caminho dev (hot reload)** — precisa do toolchain Android na máquina:

```bash
# pré-requisitos (uma vez): JDK 17, Android SDK + NDK, e:
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
export JAVA_HOME=<jdk17> ANDROID_HOME=<sdk> NDK_HOME=<sdk>/ndk/<versão>

npm install
npm run tauri android dev        # escolhe device/emulador e roda
```

> No Linux da Blue3 o toolchain já existe (`~/Android/jdk-17`, `~/Android/Sdk`,
> NDK 28.2) — o ambiente está em `docs/build.md`/histórico do repo.

**O que conferir no aparelho (0.7.4, ainda não rodou em aparelho nenhum):**

| # | O quê | Resultado esperado |
|---|---|---|
| A1 | Toque no microfone do chat (ditado) | O Android pergunta "Permitir que o ShvIA grave áudio?"; permitido, grava e transcreve. Antes da 0.7.4 não perguntava e nada acontecia |
| A2 | Anexar foto, escolhendo a câmera | O Android pergunta pela câmera; permitido, abre e anexa |
| A3 | Negar a permissão | O app continua abrindo e o resto funciona; só aquele recurso fica sem |
| A4 | Aparelho sem câmera ou sem microfone (emulador sem os dois) | O app **instala**: as duas `uses-feature` são `required="false"` |

O CI (`.github/workflows/android.yml`) compila o APK de depuração e confere as permissões dentro dele,
mas só quando o app muda; ele não substitui estes quatro passos, que precisam de um aparelho.

---

## iOS (só macOS) — este passo É o começo do M2

```bash
# pré-requisitos (uma vez):
xcode-select --install                    # ou Xcode completo da App Store
brew install cocoapods
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

npm install
npm run tauri ios init                    # gera src-tauri/gen/apple (COMMITAR!)
npm run tauri icon brand/shvia-desktop-icon-1024.png   # preenche os ícones no gen/apple recém-criado
npm run tauri ios dev                     # escolhe simulador; 1º build demora
```

- **Signing já pré-configurado** (0.2.6): o `tauri ios init` gera o projeto com o
  **Team ID `S65UBCTPN5`** (`bundle.iOS.developmentTeam` no `tauri.conf.json`).
  Pré-requisito no Mac: Xcode logado na conta Apple da Blue3 (Settings ▸ Accounts).
  Se o Xcode reclamar de assinatura, abrir `src-tauri/gen/apple/*.xcodeproj` ▸
  Signing & Capabilities e conferir o time.
- **Permissões de mic/câmera** já entram via `src-tauri/Info.ios.plist` (0.2.6, o
  Tauri mescla no Info.plist gerado) — o teste 5 (ditado) deve mostrar o prompt do
  iOS com o texto em português; se o app fechar sozinho ao tocar o mic, a mescla
  não aconteceu.
- **Device físico:** `npm run tauri ios dev --host` ou pelo próprio Xcode.
- Ícones já gerados em `src-tauri/icons/ios/` (0.2.2); o `tauri icon` acima os
  aplica ao projeto Xcode.
- Rodou → **commitar o `gen/apple`** do MacBook (mesmo padrão do `gen/android`),
  com bump de versão (Y: novo alvo de runtime).

---

## Truques

- `?hold` / `?hold=offline` na casca (dev no navegador) congelam os estados do
  splash pra estilização — `npm run dev` + `localhost:1420/?hold=offline`.
- Streaming SSE parece "tudo de uma vez"? Conferir se não há proxy/VPN bufferizando;
  testar em 4G puro também.

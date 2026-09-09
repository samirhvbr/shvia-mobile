# ShvIA Mobile

Cliente **mobile (iOS + Android)** do ShvIA — **shell fino Tauri 2** que carrega o
ShvIA hospedado (`https://ai.shvia.org`) na WebView nativa (WKWebView / Android
System WebView). Irmão do `SHVIA-DESKTOP` (desktop) e do `SHVIA` (servidor Laravel).

> Antes de mexer: **`git pull`**. Convenções em [CLAUDE.md](CLAUDE.md) ·
> Roadmap em [.continue/escopo-mobile.md](.continue/escopo-mobile.md) ·
> Decisões em [docs/decisoes.md](docs/decisoes.md).
>
> **Todos os documentos:** [decisoes.md](docs/decisoes.md) (ADRs) ·
> [testflight-checklist.md](docs/testflight-checklist.md) (ordem de execução da
> publicação) · [loja-ficha.md](docs/loja-ficha.md) (campos do App Store Connect,
> prontos para colar) · [smoke-test.md](docs/smoke-test.md) (roteiro manual no
> aparelho) · [app-review-notes.md](docs/app-review-notes.md) (o texto das Notes
> do App Store Connect, com a revisão de 09/09/2026) · [rejeicao-20260812.md](docs/rejeicao-20260812.md)
> (a mensagem literal da App Review). O `docs/` deste repo é pequeno o bastante para caber aqui — não há
> `docs/README.md` separado, e o teste `todo_doc_e_alcancavel` reprova documento
> que fique de fora desta lista.

## Status

**M2 fechado** (15/07): o app roda no **iPhone físico** via TestFlight interno,
com biometria (Face ID/Touch ID) na 0.4.0 e o domínio próprio `ai.shvia.org` na
0.5.0. Android **builda** (APK/AAB no Linux), mas ainda não rodou em aparelho.

**Em publicação (revisão de 30/07).** O caminho crítico agora é o M4. Ordem de
execução no **[docs/testflight-checklist.md](docs/testflight-checklist.md) §0**;
o que a revisão achou, em [.continue/escopo-mobile.md](.continue/escopo-mobile.md) §5.
Em uma linha:

- 🟢 **Privacidade e suporte:** escritas em 30/07 no `SHVIA-SITE` 0.4.0
  (`shvia.org/privacidade.html` e `/suporte.html`) — era o bloqueio duro do App
  Store Connect. Falta razão social + CNPJ, as caixas de e-mail e o deploy.
- 🔴 **`gen/apple` é de 14/07** e ficou defasado: faltava `NSFaceIDUsageDescription`
  (o iOS **encerra o app** sem ela, no card de Face ID da 1ª execução), faltava
  `ITSAppUsesNonExemptEncryption`, a versão estava em `0.2.6` e o
  `PrivacyInfo.xcprivacy` não está no alvo Xcode. Remendado na 0.5.3; a cura é
  `tauri ios init` no Mac.
- 🟡 **Push (APNs) segue 0% no cliente** — servidor pronto desde a 2.51.0, mas nem
  a casca nem o front do SHVIA-WEB registram token. É a defesa mais forte contra a
  regra **4.2** e a aposta em aberto da submissão pública.

## Build

```bash
npm install
# valida o Rust no host (sem toolchain mobile):
cargo check --manifest-path src-tauri/Cargo.toml
# Android (precisa Android SDK/NDK + JDK 17):
npm run tauri android init && npm run tauri android dev
# iOS (precisa macOS + Xcode):
npm run tauri ios init && npm run tauri ios dev
```

## Arquitetura

Shell fino: a WebView navega o FQDN real; o **servidor Laravel é a fonte da
verdade** (dados, auth por **cookie de sessão same-origin**). Nenhum banco nem
segredo moram no cliente. Links externos (fora dos hosts do servidor — `SERVER_HOSTS` em `src-tauri/src/lib.rs`) abrem no
navegador do SO. Tarja "Sistema Offline" injetada em cada página.

## Produção / lojas

Reusa a **infra da Blue3** (que já publica o app Flutter `BLUE3-INTRANET-MOBILE`):
Apple Developer **Team ID `S65UBCTPN5`**, Google Play Console, custódia de keystore.
Bundle ID **`cloud.blue3.shvia`** (mesmo do desktop). Detalhes e fases no roadmap.

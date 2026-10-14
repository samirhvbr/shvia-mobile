#!/usr/bin/env node
// Keeps the app icons on ONE identity. Two jobs:
//   node scripts/verify-icons.mjs           check (CI: `npm run prova:icones`)
//   node scripts/verify-icons.mjs --selar   re-stamp brand/icones.json after a deliberate export
//
// Why it exists: ShvIA changed identity three times in three weeks (asa "AI" -> Portal 16/09 ->
// Shield 27/09). The exporter in SHVIA-WEB `brand/tools/export.cjs` was run for the web and the
// desktop but not for this repo, so the 0.7.7 IPA was one export behind the approved brand and
// nothing noticed: the CI compiles no iOS and the icons are binary. This cannot read SHVIA-WEB
// (a sibling checkout the CI does not have), so it pins what this repo holds:
//   1. every icon file is byte-identical to what was stamped (a stray old export turns it red);
//   2. the source SVG is the Shield, never the Portal;
//   3. iOS icons carry no alpha channel (App Store Connect refuses the upload);
//   4. the Android adaptive background is the Shield's ink, not Portal's cyan;
//   5. the app's local screen uses the same mark as the stamped symbol.
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const raiz = join(dirname(fileURLToPath(import.meta.url)), '..');
const MANIFESTO = join(raiz, 'brand/icones.json');
const RES = 'src-tauri/gen/android/app/src/main/res';
const PASTAS = [
  'src-tauri/icons',
  'src-tauri/icons/ios',
  'src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset',
  ...['mdpi', 'hdpi', 'xhdpi', 'xxhdpi', 'xxxhdpi'].map((d) => `${RES}/mipmap-${d}`),
];
const PASTAS_IOS = ['src-tauri/icons/ios', 'src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset'];
const FONTE = 'brand/shield-app.svg';
const EXTRAS = ['brand/shield-app.svg', 'brand/shield-symbol.svg', 'brand/shvia-desktop-icon-1024.png', 'brand/shvia-mark.png', 'public/brand-mark.svg'];
const TINTA = '#111317';

const sha = (rel) => createHash('sha256').update(readFileSync(join(raiz, rel))).digest('hex');
const arquivos = () => {
  const out = [...EXTRAS];
  for (const p of PASTAS) {
    for (const n of readdirSync(join(raiz, p)).sort()) {
      if (/\.(png|ico|icns)$/.test(n)) out.push(`${p}/${n}`);
    }
  }
  return [...new Set(out)].sort();
};

if (process.argv.includes('--selar')) {
  const fonte = readFileSync(join(raiz, FONTE), 'utf8');
  const m = {
    identidade: 'shield',
    aprovada_em: '2026-09-27',
    origem: 'SHVIA-WEB brand/atual (app-icon-mobile.svg), exportada por brand/tools/export.cjs',
    nota: 'Selo gerado por `npm run icones:selar` depois de uma exportacao deliberada. Nao edite a mao.',
    fonte_svg: FONTE,
    fonte_sha256: createHash('sha256').update(fonte).digest('hex'),
    arquivos: Object.fromEntries(arquivos().map((f) => [f, sha(f)])),
  };
  writeFileSync(MANIFESTO, JSON.stringify(m, null, 2) + '\n');
  console.log(`[icones] selo gravado: ${Object.keys(m.arquivos).length} arquivos`);
  process.exit(0);
}

const erros = [];
if (!existsSync(MANIFESTO)) {
  console.error('[icones] brand/icones.json ausente: rode `npm run icones:selar` depois de exportar os icones.');
  process.exit(1);
}
const m = JSON.parse(readFileSync(MANIFESTO, 'utf8'));
if (m.identidade !== 'shield') erros.push(`o selo diz "${m.identidade}", e a identidade aprovada e "shield"`);

// 1. byte-identical to the stamp, and nothing added or missing
const atuais = arquivos();
for (const f of atuais) {
  if (!(f in m.arquivos)) erros.push(`${f}: arquivo novo fora do selo (exporte e rode icones:selar)`);
  else if (sha(f) !== m.arquivos[f]) erros.push(`${f}: difere do selo (outra exportacao? rode icones:selar so se foi proposital)`);
}
for (const f of Object.keys(m.arquivos)) if (!atuais.includes(f)) erros.push(`${f}: no selo mas ausente`);

// 2. the source is the Shield
const svg = readFileSync(join(raiz, FONTE), 'utf8');
if (createHash('sha256').update(svg).digest('hex') !== m.fonte_sha256) erros.push(`${FONTE}: difere do selo`);
for (const marca of ['ShvIA mobile app icon', TINTA, 'M32 4C40 8.7']) {
  if (!svg.includes(marca)) erros.push(`${FONTE}: nao traz "${marca}" (nao parece o Shield)`);
}
if (/34B3EC/i.test(svg)) erros.push(`${FONTE}: traz o ciano #34B3EC do Portal`);

// 3. no alpha in the iOS icons (PNG colour type 6/4 or a tRNS chunk)
for (const p of PASTAS_IOS) {
  for (const n of readdirSync(join(raiz, p)).filter((x) => x.endsWith('.png'))) {
    const b = readFileSync(join(raiz, p, n));
    const tipo = b[25];
    if (tipo === 6 || tipo === 4 || b.includes(Buffer.from('tRNS'))) erros.push(`${p}/${n}: tem canal alfa (a App Store recusa)`);
  }
}

// 4. Android adaptive background
const fundo = readFileSync(join(raiz, `${RES}/values/ic_launcher_background.xml`), 'utf8');
if (!fundo.toLowerCase().includes(TINTA)) erros.push(`${RES}/values/ic_launcher_background.xml: o fundo nao e ${TINTA}`);
if (/34B3EC/i.test(fundo)) erros.push(`${RES}/values/ic_launcher_background.xml: ainda e o ciano do Portal`);

// 4b. no Portal cyan left in any Android resource, referenced or not
const xmls = (d) => readdirSync(join(raiz, d), { withFileTypes: true }).flatMap((e) =>
  e.isDirectory() ? xmls(`${d}/${e.name}`) : e.name.endsWith('.xml') ? [`${d}/${e.name}`] : []);
for (const f of xmls(RES)) {
  if (/34B3EC/i.test(readFileSync(join(raiz, f), 'utf8'))) erros.push(`${f}: ainda traz o ciano #34B3EC do Portal`);
}

// 5. the local screen uses the stamped symbol
if (sha('public/brand-mark.svg') !== sha('brand/shield-symbol.svg')) erros.push('public/brand-mark.svg difere de brand/shield-symbol.svg');

if (erros.length) {
  console.error(`[icones] ${erros.length} problema(s):\n  - ${erros.join('\n  - ')}`);
  process.exit(1);
}
console.log(`[icones] ok: identidade shield, ${atuais.length} arquivos conferem com o selo.`);

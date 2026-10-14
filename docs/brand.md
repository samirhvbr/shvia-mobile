# Shield identity

The approved identity is the **Shield** (a bipartite graphite and titanium shield with six orchestration nodes), approved on 27/09/2026. Its artwork lives in the SHVIA-WEB repository under `brand/atual`; `brand/versao2-portal` keeps the Portal that preceded it and `brand/versao1` the identity before that. This repository carried the **Portal** (cyan `#34B3EC`, navy `#0B0F17`) from 0.6.32 (16/09) until 0.7.8, one export behind the brand: the web and the desktop were exported, this repository was not, and the 0.7.7 IPA was built with the old icon.

Palette: cobalt `#2350E6`, graphite `#2A2D32`, titanium `#BFC4CA`, deep blue `#0E2468`, light blue `#6E97FF`, mist `#F4F5F7`, ink `#111317`. No gradients or glow.

## What is where

- **iOS:** `src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset/` (18 files) and `src-tauri/icons/ios/`, all opaque (no alpha: App Store Connect refuses it). Source: `brand/shield-app.svg`, the opaque ink square; the system rounds the corners.
- **Android:** `mipmap-*dpi/ic_launcher*.png` for the legacy icon, and the adaptive pair `ic_launcher_foreground.png` (transparent, safe centre zone) over the background `#111317` in `values/ic_launcher_background.xml`. The result is the iOS composition.
- **The app's own screens** (bootstrap, offline, lock): `public/brand-mark.svg`, the Shield symbol.
- **The seal:** `brand/icones.json` records the sha256 of every icon file above and of the source SVG. `npm run prova:icones` (a CI step) fails when a file differs from the seal, when the source is not the Shield, when an iOS icon has an alpha channel, or when Portal cyan is anywhere in the Android resources.

## Refreshing the icons

1. Check that the sibling checkouts are clean: the exporter writes into SHVIA-WEB, the desktop, this repository, the site and the Workspace at once, and the web part resolves its paths from the script's own location. **Run it on a copy** (copy `brand/`, `public/` and the icon folders of each repo into a scratch root, run there, diff, and bring over only what belongs to this repository).
2. `NODE_PATH=<sharp modules> node brand/tools/export.cjs <root>`, then `brand/tools/export-native.py <root>` with Python and Pillow (`sharp` and Pillow are not installed on the build Mac; a venv and an `npm install sharp` outside the repositories are enough).
3. **The Android adaptive foreground is the one exception.** The exporter paints the shield's cobalt in ink (`#111317`), a leftover of the Portal's dark-on-cyan foreground, which suits a light background and makes the accents vanish on the Shield's ink background. Render the foreground from `shield-symbol.svg` in its own colours, in the same safe-centre transform, at the sizes already in `mipmap-*dpi` (108, 162, 216, 324, 432 px). Fixing the exporter in SHVIA-WEB (drop the `replaceAll('#2350E6', …)`) would remove the exception.
4. `npm run icones:selar`, then `npm run prova:icones`.

Updating source assets does not replace installed applications or submit store builds. Rebuild and distribute through the existing release process. The App Store shows the previous icon until a build carrying the new one is approved. Site social images are rendered by `bin/make-images.sh` in the site repository after editing its OG templates.

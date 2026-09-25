fn main() {
    // App commands under the ACL (ADR-005). Without this manifest, app commands are callable
    // from any LOCAL page and never from a remote one; with it, each command is a permission
    // (`allow-<name>`) that a capability has to grant, and the `servidor-shvia` capability can
    // grant `salvar_arquivo` to the ShvIA hosts without granting anything else.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["trava_biometrica", "salvar_arquivo"]),
    ))
    .expect("tauri-build failed");
}

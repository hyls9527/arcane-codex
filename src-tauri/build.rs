fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest::embed_manifest(embed_manifest::new_manifest("Arcane.Codex"))
            .expect("unable to embed manifest file");
    }
    println!("cargo:rerun-if-changed=build.rs");
    tauri_build::build()
}

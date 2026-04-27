fn main() {
    // For tests and library compilation, skip full Tauri build.
    // Full Tauri build with icon generation runs during `cargo build --bin`.
    // This allows `cargo test --lib` and `cargo test --test` to run independently.
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    {
        let mut res = tauri_build::WindowsAttributes::new();
        // Add Common Controls v6 manifest to fix TaskDialogIndirect error
        res = res.app_manifest(
            r#"
            <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
              <dependency>
                <dependentAssembly>
                  <assemblyIdentity
                    type="win32"
                    name="Microsoft.Windows.Common-Controls"
                    version="6.0.0.0"
                    processorArchitecture="*"
                    publicKeyToken="6595b64144ccf1df"
                    language="*"
                  />
                </dependentAssembly>
              </dependency>
            </assembly>
            "#,
        );
        tauri_build::try_build(
            tauri_build::Attributes::new().windows_attributes(res),
        )
        .expect("failed to run tauri-build");
    }
}

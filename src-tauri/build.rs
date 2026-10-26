fn main() {
    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let rc_path = std::path::Path::new(&out_dir).join("resource.rc");
        let obj_path = std::path::Path::new(&out_dir).join("resource_direct.o");

        if rc_path.exists() {
            let status = std::process::Command::new("windres")
                .args(&[
                    "--input",
                    rc_path.to_str().unwrap(),
                    "--output",
                    obj_path.to_str().unwrap(),
                ])
                .status();

            if let Ok(s) = status {
                if s.success() && obj_path.exists() {
                    println!("cargo:rustc-link-arg={}", obj_path.display());
                }
            }
        }
    }
}

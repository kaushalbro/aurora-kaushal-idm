use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources/resources.rc");
    println!("cargo:rerun-if-changed=resources/icon.ico");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let out_dir = env::var("OUT_DIR").unwrap();
        let res_file = Path::new(&out_dir).join("resources.o");

        let windres_cmd = env::var("WINDRES").unwrap_or_else(|_| {
            if Command::new("x86_64-w64-mingw32-windres").output().is_ok() {
                "x86_64-w64-mingw32-windres".to_string()
            } else {
                "windres".to_string()
            }
        });

        let status = Command::new(windres_cmd)
            .args(["-I", "resources"])
            .args(["-i", "resources/resources.rc"])
            .args(["-o", res_file.to_str().unwrap()])
            .status();

        if let Ok(st) = status {
            if st.success() {
                println!("cargo:rustc-link-arg={}", res_file.to_str().unwrap());
            }
        }
    }
}

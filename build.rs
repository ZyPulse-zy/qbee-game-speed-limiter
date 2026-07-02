use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app-icon.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let resource = out_dir.join("download_client_app_icon.o");
    let windres = env::var("WINDRES").unwrap_or_else(|_| "windres".to_string());

    let status = Command::new(&windres)
        .current_dir("assets")
        .args(["app.rc", "-O", "coff", "-o"])
        .arg(&resource)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("cargo:rustc-link-arg-bins={}", resource.display());
        }
        Ok(status) => {
            println!(
                "cargo:warning=windres exited with status {status}; Windows exe icon was not embedded"
            );
        }
        Err(error) => {
            println!(
                "cargo:warning=windres not available ({error}); Windows exe icon was not embedded"
            );
        }
    }
}

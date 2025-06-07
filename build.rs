fn main() {
    // Read the TAURI_PLATFORM environment variable
    if let Ok(platform) = std::env::var("TAURI_PLATFORM") {
        match platform.as_str() {
            "android" => {
                println!("cargo:rustc-cfg=feature=\"mobile\"");
                println!("cargo:rustc-cfg=feature=\"android\"");
            }
            "ios" => {
                println!("cargo:rustc-cfg=feature=\"mobile\"");
                println!("cargo:rustc-cfg=feature=\"ios\"");
            }
            "desktop" => {
                println!("cargo:rustc-cfg=feature=\"desktop\"");
            }
            _ => {}
        }
    }

    // Tell cargo to rerun this script if TAURI_PLATFORM changes
    println!("cargo:rerun-if-env-changed=TAURI_PLATFORM");
}
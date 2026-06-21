fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let mut builder = cc::Build::new();
    builder
        .cpp(true)
        // The event-tap callback runs on every keystroke. Keep the native
        // engine optimized for speed even when the Rust shell uses
        // `opt-level = "z"` to reduce the final Tauri bundle size.
        .opt_level(2)
        .file("engine/ConvertTool.cpp")
        .file("engine/Engine.cpp")
        .file("engine/EnglishDictionary.cpp")
        .file("engine/Macro.cpp")
        .file("engine/SmartSwitchKey.cpp")
        .file("engine/Vietnamese.cpp")
        .file("src/engine_wrapper.cpp")
        .include("engine");

    println!("cargo:rerun-if-changed=engine/ConvertTool.cpp");
    println!("cargo:rerun-if-changed=engine/Engine.cpp");
    println!("cargo:rerun-if-changed=engine/EnglishDictionary.cpp");
    println!("cargo:rerun-if-changed=engine/Macro.cpp");
    println!("cargo:rerun-if-changed=engine/SmartSwitchKey.cpp");
    println!("cargo:rerun-if-changed=engine/Vietnamese.cpp");
    println!("cargo:rerun-if-changed=src/engine_wrapper.cpp");

    if target_os == "linux" {
        builder.define("LINUX", None);
    }

    builder.compile("vnkey_engine");

    if target_os == "macos" {
        // Compile the Objective-C++ adapter separately so ARC applies only to
        // Cocoa objects, not to the portable C++ engine sources. The adapter
        // keeps cached NSString/NSDictionary objects across event callbacks;
        // ARC is required to keep those references alive safely.
        cc::Build::new()
            .cpp(true)
            .opt_level(2)
            .flag("-mmacosx-version-min=11.0")
            .flag("-fobjc-arc")
            .file("src/tauri_event_tap.mm")
            .include("engine")
            .compile("vnkey_macos_adapter");
        println!("cargo:rerun-if-changed=src/tauri_event_tap.mm");
        println!("cargo:rustc-link-lib=framework=Carbon");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }

    tauri_build::build();
}

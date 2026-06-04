fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let amd_feature = std::env::var("CARGO_FEATURE_AMD").is_ok();

    if amd_feature && target_arch == "x86_64" {
        println!("cargo:warning=Building AMD SMI FFI bridge (x86_64)");

        cxx_build::bridge("src/lib.rs")
            .file("cpp/amd_smi_wrapper.cpp")
            .include("cpp")
            .include("amd_smi")
            .flag_if_supported("-std=c++17")
            .flag_if_supported("-Wall")
            .flag_if_supported("-Wextra")
            .compile("amd_smi_bridge");

        println!("cargo:rustc-link-search=native=.");
        println!("cargo:rustc-link-lib=static=amd_smi");

        println!("cargo:rerun-if-changed=cpp/amd_smi_wrapper.cpp");
        println!("cargo:rerun-if-changed=cpp/amd_smi_wrapper.h");
    } else if amd_feature {
        println!("cargo:warning=AMD feature enabled but not x86_64 — skipping C++ build");
    }

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
}

fn main() {
    // AMD SMI C++ compilation is now handled by the is-amd-ffi crate.
    // This build.rs is kept minimal.
    println!("cargo:rerun-if-changed=build.rs");
}

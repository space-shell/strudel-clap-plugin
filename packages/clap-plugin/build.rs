fn main() {
    // Ensure JavaScript bundle is re-embedded when it changes
    println!("cargo:rerun-if-changed=bundled/strudel-runtime.js");
    println!("cargo:rerun-if-changed=build.rs");

    // Verify that the bundle exists at build time
    let bundle_path = std::path::Path::new("bundled/strudel-runtime.js");
    if !bundle_path.exists() {
        panic!(
            "JavaScript bundle not found at {:?}. Run ./scripts/bundle-runtime.sh first.",
            bundle_path
        );
    }
}

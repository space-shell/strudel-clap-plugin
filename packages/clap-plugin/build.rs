fn main() {
    // Build script placeholder
    // nih-plug handles symbol exports automatically for basic plugins
    println!("cargo:rerun-if-changed=build.rs");
}

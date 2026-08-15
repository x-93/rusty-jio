fn main() {
    // Expose git commit hash or build metadata to compiler
    println!("cargo:rerun-if-changed=build.rs");
}

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/asm/asm.S");
    println!("cargo:rerun-if-changed=src/lds/link.x");
    println!("cargo:rerun-if-changed=src/lds/memory.x");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.cargo/config");
}
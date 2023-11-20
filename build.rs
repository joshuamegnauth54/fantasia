fn main() {
    // Recompile if migrations change
    println!("cargo:rerun-if-changed=migrations");
}

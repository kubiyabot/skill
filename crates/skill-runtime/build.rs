// Generate WIT bindings at compile time
fn main() {
    // Tell cargo to rerun if WIT files change
    println!("cargo:rerun-if-changed=../../wit");
}

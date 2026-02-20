fn main() {
    // Ensure linkall.x is included as the final linker script.
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

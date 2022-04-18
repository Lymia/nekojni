fn main() {
    let ver = rustc_version::version_meta().unwrap();
    println!("cargo:rustc-env=RUSTC_VERSION_INFO={}", ver.short_version_string);
}

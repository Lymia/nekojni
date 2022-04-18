#![allow(deprecated)]

mod native_loader;

use std::path::PathBuf;

fn main() {
    for bin in &[
        "target/i686-pc-windows-gnu/release/examples/test_classes.dll",
        "target/x86_64-pc-windows-gnu/release/examples/test_classes.dll",
        "target/i686-unknown-linux-gnu/release/examples/libtest_classes.so",
        "target/x86_64-unknown-linux-gnu/release/examples/libtest_classes.so",
    ] {
        let binary = native_loader::ParsedBinary::parse(PathBuf::from(bin)).unwrap();
        println!("{:#?}", binary);
        println!("{:#?}", binary.load().unwrap());
    }
}

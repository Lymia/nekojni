#![allow(deprecated)]

mod build_jar;
mod native_loader;

use crate::build_jar::BuildJarOptions;
use std::path::PathBuf;

fn main() {
    let mut vec = Vec::new();
    for bin in &[
        "target/i686-pc-windows-gnu/release/examples/test_classes.dll",
        "target/x86_64-pc-windows-gnu/release/examples/test_classes.dll",
        "target/i686-unknown-linux-gnu/release/examples/libtest_classes.so",
        "target/x86_64-unknown-linux-gnu/release/examples/libtest_classes.so",
    ] {
        let binary = native_loader::ParsedBinary::parse(PathBuf::from(bin)).unwrap();
        println!("{:#?}", binary);
        println!("{:#?}", binary.load().unwrap());
        vec.push(binary);
    }

    let class_data = build_jar::make_jar_data(&vec, &BuildJarOptions {}).unwrap();
    std::fs::write("test.jar", class_data.make_jar()).unwrap();
}

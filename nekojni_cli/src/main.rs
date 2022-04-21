#![allow(deprecated)]

mod build_jar;
mod native_loader;

use crate::build_jar::BuildJarOptions;
use std::path::PathBuf;

fn main() {
    let mut vec = Vec::new();
    for bin in &[
        "target/release/examples/test_classes.dll",
        "target/release/examples/libtest_classes.so",
    ] {
        let path = PathBuf::from(bin);
        if !path.exists() {
            continue;
        }

        let binary = native_loader::ParsedBinary::parse(path).unwrap();
        println!("{:#?}", binary);
        println!("{:#?}", binary.load().unwrap());
        vec.push(binary);
    }

    let class_data =
        build_jar::make_jar_data(&vec, &BuildJarOptions { main_bin: None, use_null_loader: false })
            .unwrap();
    std::fs::write("test.jar", class_data.make_jar()).unwrap();
}

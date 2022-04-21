./compile_java.ps1

cargo build --release --example test_classes
cargo run --package nekojni_cli

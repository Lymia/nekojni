use nekojni_signatures::*;

fn main() {
    println!("{:#?}", MethodSig::parse_java("void method(int p1, short p2, java.lang.String[] p3)").unwrap());
}
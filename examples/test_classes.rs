use nekojni::*;

pub struct TestClass {
    counter: usize,
}

#[jni_export]
impl TestClass {
    pub fn increment(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}

fn main() {}

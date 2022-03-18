use nekojni::*;

pub struct TestClass {
    counter: usize,
}

#[jni_export]
impl TestClass {
    pub fn increment_foo(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
    pub fn increment_bar(self: &mut JniRef<Self>) -> usize {
        self.counter += 1;
        self.counter
    }
}

fn main() {}

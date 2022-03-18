use jni::JNIEnv;
use nekojni::*;

pub struct TestClass {
    counter: u32,
}

#[jni_export]
impl TestClass {
    pub extern "Java" fn test_func(self: &JniRef<Self>, a: u32, b: u32, c: u32) -> u32 {}
    pub extern "Java" fn test_func_2(self: &JniRef<Self>, a: u32) {}
    pub extern "Java" fn test_func_3(env: &JNIEnv, a: u32) {}

    pub fn increment_foo(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }
    pub fn increment_bar(self: &mut JniRef<Self>) -> u32 {
        self.counter += 1;
        self.counter
    }
}

fn main() {}

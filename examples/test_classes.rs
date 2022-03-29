#![feature(arbitrary_self_types)]

use jni::JNIEnv;
use nekojni::*;

pub struct TestClass {
    counter: u32,
}

#[jni_export]
#[jni(package = "moe.lymia")]
impl TestClass {
    pub extern "Java" fn test_func(self: &JniRef<Self>, a: u32, b: u32, c: u32) -> u32 {}
    pub extern "Java" fn test_func_2(self: &JniRef<Self>, a: u32) {}
    pub extern "Java" fn test_func_3(env: &JNIEnv, a: u32) {}
    pub extern "Java" fn test_func_4(self: &JniRef<Self>, a: u32, b: u32, c: u32) -> Result<u32> {}
    pub extern "Java" fn test_func_5(env: JNIEnv, a: u32) {}

    pub fn increment_foo(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }
    pub fn increment_bar(self: &mut JniRef<Self>) -> u32 {
        self.counter
    }
}

jni_module!(JniModule, "moe.lymia.JniModule");

fn main() {}

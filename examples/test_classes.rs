#![feature(arbitrary_self_types)]

use nekojni::*;

pub struct TestClass {
    counter: u32,
}

#[jni_export]
#[jni(package = "moe.lymia.test", extends = "java.lang.Thread")]
impl TestClass {
    pub extern "Java" fn test_func(self: &JniRef<Self>, a: u32, b: u32, c: u64) -> u32 {}
    pub extern "Java" fn test_func_2(self: &JniRef<Self>, a: u32) {}
    pub extern "Java" fn test_func_3(env: JniEnv, a: u32) {}
    pub extern "Java" fn test_func_4(self: &JniRef<Self>, a: u32, b: u32, c: u32) -> Result<u32> {}
    pub extern "Java" fn test_func_5(env: JniEnv, a: u32) {}
    pub extern "Java" fn test_func_6(env: JniEnv, a: u32) -> u32 {}

    pub fn increment_foo(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }

    pub fn increment_foo_x(&mut self, x: u32, y: u32, z: u32) -> u32 {
        self.counter += x + y * z;
        self.counter
    }

    pub fn increment_foo_m(self: &mut JniRefMut<Self>, x: u32, y: &u32, z: &mut u32) -> u32 {
        self.counter += x + (*y) * (*z);
        self.counter
    }

    #[jni(open)]
    pub fn increment_bar(self: &JniRef<Self>) -> u32 {
        self.counter
    }

    pub fn test_fn(self: &JniRef<Self>) -> Result<u32> {
        println!("{}", self.test_func(1, 4, 8));
        println!("{}", System::get_property(self.env(), "java.home")?);
        Ok(self.increment_bar())
    }
}

pub struct System;
#[jni_import]
#[jni(package = "java.lang")]
impl System {
    pub extern "Java" fn get_property(env: JniEnv, prop: &str) -> Result<String> {}
}

jni_module!(
    JniModule,
    "moe.lymia.nekojni.test.JniInit",
    "moe.lymia.nekojni.test.JniException"
);

#![feature(arbitrary_self_types)]

use nekojni::{objects::JArray, *};

pub struct TestClass {
    counter: u32,
}

#[jni_export]
#[jni(package = "moe.lymia.nekojni.test", extends = "java.lang.Thread")]
impl TestClass {
    /*#[jni(constructor)]
    pub extern "Java" fn new<'env>(
        env: JniEnv<'env>,
        a: u32,
        b: String,
        c: u64,
    ) -> Result<JniRef<'env, Self>> {}*/

    pub extern "Java" fn test_func(self: &JniRef<Self>, a: u32, b: u32, c: u64) -> u32 {}
    pub extern "Java" fn test_func_2(self: &JniRef<Self>, a: u32) {}
    pub extern "Java" fn test_func_3(env: JniEnv, a: u32) {}
    pub extern "Java" fn test_func_4(self: &JniRef<Self>, a: u32, b: u32, c: u32) -> Result<u32> {}
    pub extern "Java" fn test_func_5(env: JniEnv, a: u32) {}
    pub extern "Java" fn test_func_6<'env>(env: JniEnv<'env>, a: &JniRef<'env, Self>) -> u32 {}

    pub fn combine<'env>(self: &mut JniRefMut<'env, Self>, other: &mut JniRef<'env, Self>) {
        self.counter += other.counter;
    }
    pub fn combine_no_lt(self: &mut JniRefMut<Self>, other: &mut JniRef<Self>) {
        self.counter += other.counter;
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
    pub fn increment_bar<'env>(self: &JniRef<'env, Self>, other: &JniRef<'env, Self>) -> u32 {
        self.counter
    }

    pub fn test_fn(self: &JniRef<Self>) -> Result<u32> {
        println!("{}", self.test_func(1, 4, 8));
        println!("{}", System::get_property(self.env(), "java.home")?);
        Ok(self.increment_bar(self))
    }
}

pub struct System;
#[jni_import]
#[jni(package = "java.lang")]
impl System {
    pub extern "Java" fn get_property(env: JniEnv, prop: &str) -> Result<String> {}
}

pub struct MainClass;

#[jni_export]
#[jni(package = "moe.lymia.nekojni.test")]
impl MainClass {
    pub fn main(env: JniEnv, _: JArray<String>) -> Result<()> {
        println!("Hello, world (from Rust)!");
        println!("Java home: {}", System::get_property(env, "java.home")?);
        jni_bail!("oh no!")
    }
}

jni_module!(
    TestClassesModule,
    "moe.lymia.nekojni.test.TestClassesInit",
    "moe.lymia.nekojni.test.TestClassesException"
);

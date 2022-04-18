#![deny(unused_must_use)]

extern crate core;

mod classfile;
mod generate_precompiled;
mod native_class_wrapper;
pub mod signatures;
mod utils;

pub use classfile::{CFlags, FFlags, MFlags};
pub use generate_precompiled::generate_shutdown_hook;
pub use native_class_wrapper::NativeClassWrapper;

use std::collections::HashMap;

pub struct ClassData {
    class_info: HashMap<String, Vec<u8>>,
    loader_name: Option<String>,
}
impl ClassData {
    pub fn new() -> Self {
        ClassData { class_info: Default::default(), loader_name: None }
    }

    pub(crate) fn add_class(&mut self, name: &str, data: impl Into<Vec<u8>>) {
        self.class_info.insert(name.to_string(), data.into());
    }

    pub fn add_null_loader(&mut self, name: &str) {
        assert!(self.loader_name.is_none());
        self.class_info
            .insert(name.to_string(), generate_precompiled::generate_null_loader(name));
    }
    pub fn add_resource_loader(
        &mut self,
        name: &str,
        crate_name: &str,
        crate_version: &str,
        image_resource_path: &str,
    ) {
        assert!(self.loader_name.is_none());
        self.class_info.insert(
            name.to_string(),
            generate_precompiled::generate_resource_loader(
                name,
                crate_name,
                crate_version,
                image_resource_path,
            ),
        );
    }
    pub fn add_module_loader(&mut self, name: &str) {
        assert!(self.loader_name.is_some());
        self.class_info.insert(
            name.to_string(),
            generate_precompiled::generate_module_init_wrapper(
                name,
                self.loader_name.as_deref().unwrap(),
            ),
        );
    }

    pub fn add_exported_class(&mut self, exported: NativeClassWrapper) {
        exported.add_to_jar(self);
    }
}

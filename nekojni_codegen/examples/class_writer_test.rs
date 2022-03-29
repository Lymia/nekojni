use nekojni_codegen::AutoloadPath;
use nekojni_signatures::ClassName;
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

fn main() {
    let mut classes = HashMap::new();
    nekojni_codegen::generate_initialization_class(
        ClassName::new(&["moe", "lymia", "princess"], "PrincessTest"),
        &mut classes,
        Some(AutoloadPath {
            resource_prefix: "moe/lymia/princess/native_bin".into(),
            native_library_name: "princessedit_native".into(),
            temp_path_name: "PrincessEdit".into(),
            version: "0.1.0".into(),
        }),
    );
    for (k, v) in classes {
        let path = PathBuf::from(k);
        let name = format!("{}.class", path.file_name().unwrap().to_str().unwrap());
        println!("Writing: {name}");
        File::create(PathBuf::from(name))
            .unwrap()
            .write_all(&v)
            .unwrap();
    }
}

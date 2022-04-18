fn replace_str_raw(data: &[u8], source_str: &[u8], target_str: &[u8]) -> Vec<u8> {
    assert!(source_str.len() <= u16::MAX as usize);
    assert!(target_str.len() <= u16::MAX as usize);

    let source_len_bytes = (source_str.len() as u16).to_be_bytes();

    let mut out = Vec::new();
    for i in 0..data.len() - source_str.len() - 2 {
        let len_bytes = &data[i..i + 2];
        let data_bytes = &data[i + 2..i + 2 + source_str.len()];
        if len_bytes == &source_len_bytes && data_bytes == source_str {
            out.extend(&data[..i]);
            out.extend(&(target_str.len() as u16).to_be_bytes());
            out.extend(target_str);
            out.extend(&data[..i + 2 + source_str.len()]);
            return out;
        }
    }
    unreachable!("Could not find source string??")
}
fn replace_str(data: &[u8], source_str: &[u8], target_str: &str) -> Vec<u8> {
    let cesu_str = cesu8::to_java_cesu8(target_str);
    replace_str_raw(data, source_str, &cesu_str)
}

pub fn generate_shutdown_hook(class_name: &str) -> Vec<u8> {
    let data = include_bytes!("moe/lymia/nekojni/ShutdownHook.class");
    replace_str(data, b"moe/lymia/nekojni/ShutdownHook", class_name)
}

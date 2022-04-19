use byteorder::{ReadBytesExt, LE};

pub struct Hasher {
    underlying: blake3::Hasher,
}
impl Hasher {
    pub fn new() -> Self {
        Hasher { underlying: blake3::Hasher::new() }
    }
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.underlying.update(data);
        self
    }
    pub fn update_str(&mut self, data: &str) -> &mut Self {
        self.underlying.update(data.as_bytes());
        self
    }

    pub fn as_u64(&self) -> u64 {
        self.underlying.finalize_xof().read_u64::<LE>().unwrap()
    }
    pub fn as_bytes(&self) -> [u8; 32] {
        *self.underlying.finalize().as_bytes()
    }
    pub fn as_hex(&self) -> String {
        self.underlying.finalize().to_hex().to_string()
    }
}

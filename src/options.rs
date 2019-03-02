#[derive(Debug)]
pub struct Options {
    pub block_size: usize,
    pub buffer_size: usize,
    pub magic: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            block_size: 4096,
            buffer_size: 1_048_576,
            magic: 0x3405_0d23_e85c_9e3au64,
        }
    }
}

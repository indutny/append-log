#[derive(Debug)]
pub struct Options {
    /// Default: 4kb (4096 bytes)
    pub block_size: usize,

    /// Default: 1mb (1048576 bytes)
    pub buffer_size: usize,

    /// Default: 8b (8 bytes)
    pub pad_size: usize,

    /// Default: 0x3405_0d23_e85c_9e3a (a random value, really)
    pub magic: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            block_size: 4096,
            buffer_size: 1_048_576,
            pad_size: 8,
            magic: 0x3405_0d23_e85c_9e3au64,
        }
    }
}

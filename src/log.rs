extern crate crc32fast;

use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

use crc32fast::Hasher as CRC32;

use crate::Error;
use crate::Options;

#[derive(Debug)]
pub struct Log {
    file: File,
    options: Options,
    buffer: Vec<u8>,
    offset: u64,
}

impl Log {
    pub fn open_default(path: &Path) -> Result<Self, Error> {
        Self::open(path, Options::default())
    }

    pub fn open(path: &Path, options: Options) -> Result<Self, Error> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let buffer = Vec::with_capacity(options.buffer_size);
        let mut log = Log {
            file,
            options,
            buffer,
            offset: 0,
        };

        log.init()?;
        Ok(log)
    }

    fn init(&mut self) -> Result<(), Error> {
        let len = self.file.metadata()?.len();

        // Find last header
        self.offset = len;

        Ok(())
    }

    pub fn append(&mut self, data: &[u8]) -> u64 {
        let len = (data.len() as u64).to_be_bytes();
        let offset = self.offset;

        let mut hash = CRC32::new();
        hash.update(data);
        let checksum: u32 = hash.finalize();
        let checksum = checksum.to_be_bytes();

        self.buffer.extend(&len);
        self.buffer.extend(&checksum);
        self.buffer.extend(data);

        self.offset += len.len() as u64;
        self.offset += checksum.len() as u64;
        self.offset += data.len() as u64;

        offset
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        let block_size = self.options.block_size as u64;
        let pad = block_size - ((self.offset + 8) % block_size);
        self.buffer.extend(std::iter::repeat(0).take(pad as usize));
        self.buffer.extend(&self.options.magic.to_be_bytes());

        self.file.write_all(&self.buffer)?;
        self.buffer.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate tempfile;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn it_should_create_log() {
        let dir = tempdir().expect("temporary directory to create");
        let log_path = dir.path().join("log.db");

        let mut log = Log::open_default(&log_path).expect("log to open");

        log.append(&[1, 2, 3]);
        log.flush().expect("flush to succeed");
    }
}

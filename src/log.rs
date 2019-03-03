extern crate crc32fast;

use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;

use crc32fast::Hasher as CRC32;

use crate::EntryIterator;
use crate::Error;
use crate::Options;

#[derive(Debug)]
pub struct Log {
    file: File,
    options: Options,
    buffer: Vec<u8>,
    offset: u64,

    // Offset of last written data
    last_data_off: u64,
}

#[derive(Debug)]
pub struct Chunk {
    pub data: Vec<u8>,
    pub next: u64,
}

impl Log {
    pub fn open_default(path: &Path) -> Result<Self, Error> {
        Self::open(path, Options::default())
    }

    pub fn open(path: &Path, options: Options) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)?;
        let buffer = Vec::with_capacity(options.buffer_size);
        let mut log = Log {
            file,
            options,
            buffer,
            offset: 0,
            last_data_off: 0,
        };

        log.init()?;
        Ok(log)
    }

    fn init(&mut self) -> Result<(), Error> {
        let len = self.file.metadata()?.len();

        // Find last marker
        self.offset = len;
        let block_size = self.options.block_size as u64;

        // Empty file - continue
        if self.offset == 0 {
            return Ok(());
        }

        if self.offset % block_size != 0 {
            return Err(Error::InvalidLength);
        }

        let mut magic: [u8; 8] = [0; 8];
        let mut last_data_off: [u8; 8] = [0; 8];

        // Check magic
        self.file
            .seek(SeekFrom::End(-((last_data_off.len() + magic.len()) as i64)))?;
        self.file.read_exact(&mut last_data_off)?;
        self.file.read_exact(&mut magic)?;

        if u64::from_be_bytes(magic) != self.options.magic {
            return Err(Error::InvalidMagic);
        }

        self.last_data_off = u64::from_be_bytes(last_data_off);

        Ok(())
    }

    pub fn last_data_off(&self) -> u64 {
        self.last_data_off
    }

    pub fn repair(&self) -> Result<(), Error> {
        Err(Error::NotImplemented)
    }

    pub fn append(&mut self, data: &[u8]) -> u64 {
        self.last_data_off = self.offset;

        // Write `data` length

        let len = (data.len() as u64).to_be_bytes();
        self.buffer.extend(&len);
        self.offset += len.len() as u64;

        // Write checksum
        let mut hash = CRC32::new();
        hash.update(data);
        let checksum: u32 = hash.finalize();
        let checksum = checksum.to_be_bytes();

        self.buffer.extend(&checksum);
        self.offset += checksum.len() as u64;

        // Write `data` itself
        self.buffer.extend(data);
        self.offset += data.len() as u64;

        // Pad
        let pad_size = self.options.pad_size as u64;
        let pad = pad_size - (self.offset % pad_size);
        self.buffer.extend(std::iter::repeat(0).take(pad as usize));
        self.offset += pad;

        self.last_data_off
    }

    pub fn read(&mut self, off: u64) -> Result<Chunk, Error> {
        let mut len: [u8; 8] = [0; 8];
        self.file.seek(SeekFrom::Start(off))?;
        self.file.read_exact(&mut len)?;
        let len = u64::from_be_bytes(len);

        let mut crc32: [u8; 4] = [0; 4];
        self.file.read_exact(&mut crc32)?;
        let crc32 = u32::from_be_bytes(crc32);

        let mut data: Vec<u8> = std::iter::repeat(0).take(len as usize).collect();
        self.file.read_exact(&mut data)?;

        let mut hash = CRC32::new();
        hash.update(&data);
        let checksum: u32 = hash.finalize();

        if checksum != crc32 {
            return Err(Error::InvalidChecksum);
        }

        let pad_size = self.options.pad_size as u64;
        let mut next = off + 12 + len;
        next += pad_size - (next % pad_size);

        Ok(Chunk { data, next })
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        // No data to flush
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Pad to the block size
        let block_size = self.options.block_size as u64;
        let pad = block_size - ((self.offset + 16) % block_size);
        self.buffer.extend(std::iter::repeat(0).take(pad as usize));
        self.offset += pad;

        // Write offset of last data
        let last_data_off = self.last_data_off.to_be_bytes();
        self.buffer.extend(&last_data_off);
        self.offset += last_data_off.len() as u64;

        // Finish with magic value
        let magic = self.options.magic.to_be_bytes();
        self.buffer.extend(&magic);
        self.offset += magic.len() as u64;

        self.file.write_all(&self.buffer)?;
        self.file.flush()?;
        self.buffer.clear();

        Ok(())
    }

    pub fn iter(&mut self) -> Result<EntryIterator, Error> {
        EntryIterator::with_log(self)
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

        // Write data
        {
            let mut log = Log::open_default(&log_path).expect("log to open");

            log.append(&[1, 2, 3]);
            log.append(&[4, 5, 6]);
            log.flush().expect("flush to succeed");
        }

        // Re-open log
        {
            let mut log = Log::open_default(&log_path).expect("log to re-open");

            assert_eq!(log.last_data_off(), 16);
            let chunk = log.read(log.last_data_off()).expect("read to succeed");
            assert_eq!(chunk.data, vec![4, 5, 6]);
            assert_eq!(chunk.next, 32);

            let chunk = log.read(0).expect("read to succeed");
            assert_eq!(chunk.next, 16);
        }
    }

    #[test]
    fn it_should_iterate() {
        let dir = tempdir().expect("temporary directory to create");
        let log_path = dir.path().join("log.db");

        // Write data
        let mut log = Log::open_default(&log_path).expect("log to open");

        log.append(&[1, 2, 3]);
        log.append(&[4, 5, 6]);
        log.append(&[7, 8, 9]);
        log.flush().expect("flush to succeed");

        {
            let mut iter = log.iter().expect("iterator to be created");

            let chunk = iter
                .next()
                .expect("1st chunk")
                .expect("1st chunk to be read");
            assert_eq!(chunk, vec![1, 2, 3]);

            let chunk = iter
                .next()
                .expect("2nd chunk")
                .expect("2nd chunk to be read");
            assert_eq!(chunk, vec![4, 5, 6]);

            let chunk = iter
                .next()
                .expect("3rd chunk")
                .expect("3rd chunk to be read");
            assert_eq!(chunk, vec![7, 8, 9]);

            assert!(iter.next().is_none());
        }
    }
}

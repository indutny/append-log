use crate::Error;
use crate::Log;

pub struct EntryIterator<'a> {
    log: &'a mut Log,
    offset: u64,
}

impl<'a> EntryIterator<'a> {
    pub fn with_log(log: &'a mut Log) -> Result<Self, Error> {
        log.flush()?;
        Ok(EntryIterator { log, offset: 0 })
    }
}

impl<'a> std::iter::Iterator for EntryIterator<'a> {
    type Item = Result<Vec<u8>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset > self.log.last_data_off() {
            return None;
        }

        match self.log.read(self.offset) {
            Ok(chunk) => {
                self.offset = chunk.next;
                Some(Ok(chunk.data))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

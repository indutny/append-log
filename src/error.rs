use std::error::Error as StdError;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    InvalidLength,
    InvalidMagic,
    InvalidChecksum,
    NotImplemented,
    IO(io::Error),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::InvalidLength => "Log file length is not a multiple of block size",
            Error::InvalidMagic => "Invalid magic value at the end of the log file",
            Error::InvalidChecksum => "Checksum mismatch during read",
            Error::NotImplemented => "Feature is not implemented yet",
            Error::IO(_) => "IO error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(err) => write!(f, "IO error: {}", err),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

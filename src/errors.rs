use packed_struct::prelude::*;
use rustix::io::Errno;

#[derive(Debug)]
pub enum Error {
    UnknownErrno,
    NotABlockDevice,
    Errno(i32),
    FailedSerialization,
    AfterEOFAccess,
}

impl From<PackingError> for Error {
    fn from(_: PackingError) -> Self {
        Self::FailedSerialization
    }
}

impl From<Errno> for Error {
    fn from(e: Errno) -> Self {
        Self::Errno(e.raw_os_error())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.raw_os_error() {
            Some(errno) => Self::Errno(errno),
            None => Self::UnknownErrno
        }
    }
}

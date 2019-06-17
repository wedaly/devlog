//! Error type returned by the devlog library.

use std::io::Error as IOError;

#[derive(Debug)]
pub enum Error {
    /// An invalid argument was passed to the command line app
    InvalidArg(&'static str),

    /// The repository contains the maximum number of log file entries,
    /// so no more can be created.
    LogFileLimitExceeded,

    /// Wraps `io::Error`
    IOError(IOError),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Error {
        Error::IOError(err)
    }
}

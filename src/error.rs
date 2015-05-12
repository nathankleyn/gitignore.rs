extern crate glob;

use std::error;
use std::fmt;

use std::io;

pub struct Error {
    repr: ErrorCause
}

#[derive(Debug)]
enum ErrorCause {
    IoError(io::Error),
    PatternError(glob::PatternError)
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error { repr: ErrorCause::IoError(error) }
    }
}

impl From<glob::PatternError> for Error {
    fn from(error: glob::PatternError) -> Error {
        Error { repr: ErrorCause::PatternError(error) }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.repr {
            ErrorCause::IoError(ref err) => err.description(),
            ErrorCause::PatternError(ref err) => err.msg
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.repr {
            ErrorCause::IoError(ref err) => Some(err as &error::Error),
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.repr {
            ErrorCause::IoError(ref err) => {
                err.fmt(f)
            },
            ErrorCause::PatternError(ref err) => {
                err.fmt(f)
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

use std::error::Error as StdError;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InvalidROM,
    EndOfROM,
    UnknownInstruction(u16),
    StackOverflow,
    StackUnderflow,
    Unexpected(Box<dyn StdError>),
}

impl StdError for Error {
    fn description(&self) -> &str {
        "description"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(ref err) => err.fmt(f),
            Error::InvalidROM => f.write_str("Invalid ROM"),
            Error::EndOfROM => f.write_str("End of ROM"),
            Error::UnknownInstruction(i) => {
                f.write_fmt(format_args!("Unknown instruction: {:#X}", i))
            }
            Error::StackOverflow => f.write_str("Stack overflow"),
            Error::StackUnderflow => f.write_str("Stack underflow"),
            Error::Unexpected(ref err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

use std::fmt;

#[derive(Debug)]
pub enum Error {
  IO(String)
}

impl std::error::Error for Error { }

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Error::IO(err.to_string())
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match &*self {
      Error::IO(s) => {
        write!(f, "I/O error; {}", s)
      }
    }
  }
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :

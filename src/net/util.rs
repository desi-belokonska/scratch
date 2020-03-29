use std::io::Error;

pub fn into_io_error(err: nix::Error) -> Error {
  Error::from(err.as_errno().unwrap())
}

use std::io::Error;

pub fn into_io_error(err: nix::Error) -> Error {
  Error::from(err.as_errno().unwrap())
}

pub fn buffer_to_str(buf: &mut [u8], up_to: usize) -> &str {
  std::str::from_utf8(&buf[..up_to]).expect("Error in str conv")
}

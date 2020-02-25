use scratch::*;
use std::io;
use std::io::{Read, Write};

fn main() -> io::Result<()> {
  let hello = "Hello from server";

  let listener = TcpListener::bind("127.0.0.1:8000")?;

  loop {
    let (mut stream, _) = listener.accept()?;
    let buffer = &mut [0; 30000];

    let bytes_read = (&stream).read(buffer)?;

    println!("{}", buffer_to_str(buffer, bytes_read));

    stream.write(hello.as_bytes())?;
    println!("-> Hello message sent");
    stream.close()?;
  }
}

fn buffer_to_str(buf: &mut [u8], up_to: usize) -> &str {
  std::str::from_utf8(&buf[..up_to]).expect("Error in str conv")
}

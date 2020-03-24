use crate::net::http::{Request, Response};
use crate::net::tcp::*;
use crate::net::util;
use std::io::Result as IoResult;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::net::ToSocketAddrs;

// const DEFAULT_MAX_HEADER_BYTES: u32 = 1 << 20;

pub struct Server {
  inner: TcpListener<Socket>,
}

impl Server {
  pub fn bind(addr: impl ToSocketAddrs) -> Self {
    let listener = TcpListener::<Socket>::bind(addr)
      .unwrap_or_else(|e| panic!("error binding to address: {}", e));
    Server { inner: listener }
  }

  pub fn serve<'a, F>(&self, handle_fn: F) -> IoResult<()>
  where
    F: Fn(Request) -> IoResult<Response>,
  {
    println!("Server listening");

    for stream in self.inner.incoming() {
      let stream = stream?;

      let mut reader = BufReader::new(&stream);
      let mut writer = BufWriter::new(&stream);

      let buffer = &mut [0; 30000];
      let bytes_read = reader.read(buffer)?;
      let raw_request = util::buffer_to_str(buffer, bytes_read).expect("Invalid request");

      let request =
        Request::parse(raw_request).map_err(|_| Error::from(ErrorKind::InvalidInput))?;

      let response = handle_fn(request)?;
      println!("👩‍🏭");
      writer.write_all(&response.as_bytes())?;
    }
    Ok(())
  }
}

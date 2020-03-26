use crate::net::http::{Request, Response};
use crate::net::tcp::*;
use std::io::Result as IoResult;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::net::ToSocketAddrs;
use std::time::SystemTime;

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
    match self.inner.local_addr() {
      Ok(addr) => info!("Server listening on {}", addr),
      Err(err) => error!("Error getting local address: {}", err),
    }

    for stream in self.inner.incoming() {
      let now = SystemTime::now();
      let stream = stream?;

      let mut reader = BufReader::new(&stream);
      let mut writer = BufWriter::new(&stream);

      let buffer = &mut [0; 30000];
      reader.read(buffer)?;
      let raw_request = String::from_utf8_lossy(&buffer[..]);

      let request =
        Request::parse(&raw_request).map_err(|_| Error::from(ErrorKind::InvalidInput))?;

      info!("{}", raw_request);

      let response = handle_fn(request)?;

      writer.write_all(&response.as_bytes())?;

      match now.elapsed() {
        Ok(elapsed) => {
          // it prints '2'
          info!(
            "took: {} microsecs ({} secs)",
            elapsed.as_micros(),
            elapsed.as_secs()
          );
        }
        Err(e) => {
          // an error occurred!
          error!("Error: {:?}", e);
        }
      }
    }
    Ok(())
  }
}

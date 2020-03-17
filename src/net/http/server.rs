use crate::net::tcp::*;
use std::io::Result as IoResult;
use std::net::ToSocketAddrs;

// const DEFAULT_MAX_HEADER_BYTES: u32 = 1 << 20;

pub struct Server {
  inner: TcpListener<Socket>,
}

impl Server {
  pub fn bind(addr: impl ToSocketAddrs) -> IoResult<Self> {
    let listener = TcpListener::<Socket>::bind(addr)?;
    Ok(Server { inner: listener })
  }

  pub fn serve(&self, handle_fn: impl Fn(TcpStream<Socket>) -> IoResult<()>) -> IoResult<()> {
    for stream in self.inner.incoming() {
      let stream = stream?;
      handle_fn(stream)?;
    }
    Ok(())
  }
}

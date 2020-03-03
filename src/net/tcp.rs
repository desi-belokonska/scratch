use super::util::into_io_error;
use nix::sys::socket::{
  accept, bind, getpeername, listen, socket, AddressFamily, InetAddr, IpAddr, SockAddr, SockFlag,
  SockProtocol, SockType,
};
use nix::unistd::{close, read, write};
use std::io::{Error, ErrorKind, Read, Result as IoResult, Write};
use std::net::{SocketAddr, ToSocketAddrs};

pub trait SocketLike {
  fn new() -> IoResult<Box<Self>>;
  fn accept(&self) -> IoResult<Box<Self>>;
  fn get_peer_name(&self) -> IoResult<SocketAddr>;
  fn bind(&mut self, addr: SocketAddr) -> IoResult<()>;
  fn listen(&self, backlog: usize) -> IoResult<()>;
  fn close(&self) -> IoResult<()>;
  fn read(&self, buf: &mut [u8]) -> IoResult<usize>;
  fn write(&mut self, buf: &[u8]) -> IoResult<usize>;
}

// ----- Begin: Socket ------

pub struct Socket(i32);

impl SocketLike for Socket {
  fn new() -> IoResult<Box<Socket>> {
    match socket(
      AddressFamily::Inet,
      SockType::Stream,
      SockFlag::empty(),
      SockProtocol::Tcp,
    ) {
      Ok(raw_fd) => Ok(Box::new(Socket(raw_fd))),
      Err(err) => Err(into_io_error(err)),
    }
  }

  fn accept(&self) -> IoResult<Box<Socket>> {
    match accept(self.0) {
      Ok(raw_fd) => Ok(Box::new(Socket(raw_fd))),
      Err(err) => Err(into_io_error(err)),
    }
  }

  fn get_peer_name(&self) -> IoResult<SocketAddr> {
    let addr = getpeername(self.0).map_err(into_io_error)?;
    match addr {
      SockAddr::Inet(iaddr) => Ok(iaddr.to_std()),
      _ => Err(Error::from(ErrorKind::Other)),
    }
  }

  fn bind(&mut self, addr: SocketAddr) -> IoResult<()> {
    let address = SockAddr::new_inet(InetAddr::new(IpAddr::from_std(&addr.ip()), addr.port()));
    bind(self.0, &address).map_err(into_io_error)
  }

  fn listen(&self, backlog: usize) -> IoResult<()> {
    listen(self.0, backlog).map_err(into_io_error)
  }

  fn close(&self) -> IoResult<()> {
    close(self.0).map_err(into_io_error)
  }

  fn read(&self, buf: &mut [u8]) -> IoResult<usize> {
    read(self.0, buf).map_err(into_io_error)
  }

  fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
    write(self.0, buf).map_err(into_io_error)
  }
}

impl Drop for Socket {
  fn drop(&mut self) {
    self.close().unwrap()
  }
}

// ----- End Socket ------

// ----- Start TcpStream ------

pub struct TcpStream<T: SocketLike> {
  inner: T,
}

impl<T: SocketLike> Read for TcpStream<T> {
  fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
    self.inner.read(buf)
  }
}

impl<T: SocketLike> Read for &TcpStream<T> {
  fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
    self.inner.read(buf)
  }
}

impl<T: SocketLike> Write for TcpStream<T> {
  fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
    self.inner.write(buf)
  }

  fn flush(&mut self) -> IoResult<()> {
    Ok(())
  }
}

// ----- End TcpStream ------

// ----- Start TcpListener ------

pub struct TcpListener<T: SocketLike> {
  inner: T,
}

impl<T: SocketLike> TcpListener<T> {
  pub fn bind(ip: impl ToSocketAddrs) -> IoResult<TcpListener<T>> {
    let ip_addresses = ip.to_socket_addrs()?;

    for addr in ip_addresses {
      if addr.is_ipv4() {
        let mut sock = *T::new()?;
        sock.bind(addr)?;
        sock.listen(128)?;
        return Ok(TcpListener { inner: sock });
      }
    }
    Err(Error::from(ErrorKind::Other))
  }

  pub fn accept(&self) -> IoResult<(TcpStream<T>, SocketAddr)> {
    let new_socket = *self.inner.accept()?;
    let socket_addr = new_socket.get_peer_name()?;
    Ok((TcpStream { inner: new_socket }, socket_addr))
  }

  pub fn incoming(&self) -> Incoming<T> {
    Incoming { listener: self }
  }
}

// ----- End TcpListener ------

pub struct Incoming<'a, T: SocketLike> {
  listener: &'a TcpListener<T>,
}

impl<'a, T: SocketLike> Iterator for Incoming<'a, T> {
  type Item = IoResult<TcpStream<T>>;

  fn next(&mut self) -> Option<IoResult<TcpStream<T>>> {
    Some(self.listener.accept().map(|p| p.0))
  }
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  struct GoodSocket {
    address: SocketAddr,
    data: Vec<u8>,
  }

  const NEW_SOCK_ADDR: &str = "127.0.0.1:3000";
  const NEW_ACCPT_ADDR: &str = "127.0.0.1:4000";

  impl SocketLike for GoodSocket {
    fn new() -> IoResult<Box<GoodSocket>> {
      // Okay because we know it's a valid socket address
      let address = NEW_SOCK_ADDR.to_socket_addrs().unwrap().next().unwrap();
      Ok(Box::new(GoodSocket {
        address,
        data: Vec::new(),
      }))
    }

    fn accept(&self) -> IoResult<Box<GoodSocket>> {
      let new_address = NEW_ACCPT_ADDR.to_socket_addrs().unwrap().next().unwrap();
      Ok(Box::new(GoodSocket {
        address: new_address,
        data: Vec::new(),
      }))
    }

    fn get_peer_name(&self) -> IoResult<SocketAddr> {
      Ok(self.address)
    }

    fn bind(&mut self, addr: SocketAddr) -> IoResult<()> {
      self.address = addr;
      Ok(())
    }

    fn listen(&self, _backlog: usize) -> IoResult<()> {
      Ok(())
    }

    fn close(&self) -> IoResult<()> {
      Ok(())
    }

    fn read(&self, buf: &mut [u8]) -> IoResult<usize> {
      let mut count = 0;
      for val in buf {
        *val = 1;
        count += 1;
      }
      Ok(count)
    }

    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
      let mut count = 0;
      for val in buf {
        self.data.push(*val);
        count += 1;
      }
      Ok(count)
    }
  }

  #[test]
  fn test_binds_tcp_listener_successfully() {
    let listener = TcpListener::<GoodSocket>::bind("127.0.0.1:1000");
    assert!(listener.is_ok());
  }

  #[test]
  fn test_accepts_incoming_connections_successfully() {
    // unwrap is okay here because we want to fail if it it's Err
    let listener = TcpListener::<GoodSocket>::bind("127.0.0.1:1000").unwrap();
    let result = listener.accept();
    assert!(result.is_ok());
    let (_, new_socket_addr) = result.unwrap();
    assert_eq!(
      new_socket_addr,
      NEW_ACCPT_ADDR.to_socket_addrs().unwrap().next().unwrap()
    )
  }
}

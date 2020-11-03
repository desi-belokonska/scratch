use super::util::into_io_error;
use nix::sys::socket::sockopt::ReuseAddr;
use nix::sys::socket::{
  accept, bind, getpeername, getsockname, listen, setsockopt, socket, AddressFamily, InetAddr,
  IpAddr, SockAddr, SockFlag, SockProtocol, SockType,
};
use nix::unistd::{close, read, write};
use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};

pub trait SocketLike {
  fn new() -> io::Result<Box<Self>>;
  fn accept(&self) -> io::Result<Box<Self>>;
  fn get_peer_name(&self) -> io::Result<SocketAddr>;
  fn get_sock_name(&self) -> io::Result<SocketAddr>;
  fn bind(&mut self, addr: SocketAddr) -> io::Result<()>;
  fn listen(&self, backlog: usize) -> io::Result<()>;
  fn close(&self) -> io::Result<()>;
  fn read(&self, buf: &mut [u8]) -> io::Result<usize>;
  fn write(&self, buf: &[u8]) -> io::Result<usize>;
}

// ----- Begin: Socket ------

pub struct Socket(i32);

impl SocketLike for Socket {
  fn new() -> io::Result<Box<Socket>> {
    let raw_fd = socket(
      AddressFamily::Inet,
      SockType::Stream,
      SockFlag::empty(),
      SockProtocol::Tcp,
    )
    .map_err(|err| into_io_error(err))?;

    setsockopt(raw_fd, ReuseAddr, &true).map_err(|err| into_io_error(err))?;

    Ok(Box::new(Socket(raw_fd)))
  }

  fn accept(&self) -> io::Result<Box<Socket>> {
    match accept(self.0) {
      Ok(raw_fd) => Ok(Box::new(Socket(raw_fd))),
      Err(err) => Err(into_io_error(err)),
    }
  }

  fn get_peer_name(&self) -> io::Result<SocketAddr> {
    let addr = getpeername(self.0).map_err(into_io_error)?;
    match addr {
      SockAddr::Inet(iaddr) => Ok(iaddr.to_std()),
      _ => Err(Error::from(ErrorKind::Other)),
    }
  }

  fn get_sock_name(&self) -> io::Result<SocketAddr> {
    let addr = getsockname(self.0).map_err(into_io_error)?;
    match addr {
      SockAddr::Inet(iaddr) => Ok(iaddr.to_std()),
      _ => Err(Error::from(ErrorKind::Other)),
    }
  }

  fn bind(&mut self, addr: SocketAddr) -> io::Result<()> {
    let address = SockAddr::new_inet(InetAddr::new(IpAddr::from_std(&addr.ip()), addr.port()));
    bind(self.0, &address).map_err(into_io_error)
  }

  fn listen(&self, backlog: usize) -> io::Result<()> {
    listen(self.0, backlog).map_err(into_io_error)
  }

  fn close(&self) -> io::Result<()> {
    close(self.0).map_err(into_io_error)
  }

  fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
    read(self.0, buf).map_err(into_io_error)
  }

  fn write(&self, buf: &[u8]) -> io::Result<usize> {
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
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.inner.read(buf)
  }
}

impl<T: SocketLike> Read for &TcpStream<T> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.inner.read(buf)
  }
}

impl<T: SocketLike> Write for TcpStream<T> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.inner.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl<T: SocketLike> Write for &TcpStream<T> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.inner.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

// ----- End TcpStream ------

// ----- Start TcpListener ------

pub struct TcpListener<T: SocketLike> {
  inner: T,
}

impl<T: SocketLike> TcpListener<T> {
  pub fn bind(ip: impl ToSocketAddrs) -> io::Result<TcpListener<T>> {
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

  pub fn accept(&self) -> io::Result<(TcpStream<T>, SocketAddr)> {
    let new_socket = *self.inner.accept()?;
    let socket_addr = new_socket.get_peer_name()?;
    Ok((TcpStream { inner: new_socket }, socket_addr))
  }

  pub fn incoming(&self) -> Incoming<T> {
    Incoming { listener: self }
  }

  pub fn local_addr(&self) -> io::Result<SocketAddr> {
    self.inner.get_sock_name()
  }
}

// ----- End TcpListener ------

pub struct Incoming<'a, T: SocketLike> {
  listener: &'a TcpListener<T>,
}

impl<'a, T: SocketLike> Iterator for Incoming<'a, T> {
  type Item = io::Result<TcpStream<T>>;

  fn next(&mut self) -> Option<io::Result<TcpStream<T>>> {
    Some(self.listener.accept().map(|p| p.0))
  }
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  static mut DATA: Vec<u8> = Vec::new();

  struct GoodSocket {
    address: SocketAddr,
  }

  const NEW_SOCK_ADDR: &str = "127.0.0.1:3000";
  const NEW_ACCPT_ADDR: &str = "127.0.0.1:4000";

  impl SocketLike for GoodSocket {
    fn new() -> io::Result<Box<GoodSocket>> {
      // Okay because we know it's a valid socket address
      let address = NEW_SOCK_ADDR.to_socket_addrs().unwrap().next().unwrap();
      Ok(Box::new(GoodSocket { address }))
    }

    fn accept(&self) -> io::Result<Box<GoodSocket>> {
      let new_address = NEW_ACCPT_ADDR.to_socket_addrs().unwrap().next().unwrap();
      Ok(Box::new(GoodSocket {
        address: new_address,
      }))
    }

    fn get_peer_name(&self) -> io::Result<SocketAddr> {
      Ok(self.address)
    }

    fn get_sock_name(&self) -> io::Result<SocketAddr> {
      Ok(self.address)
    }

    fn bind(&mut self, addr: SocketAddr) -> io::Result<()> {
      self.address = addr;
      Ok(())
    }

    fn listen(&self, _backlog: usize) -> io::Result<()> {
      Ok(())
    }

    fn close(&self) -> io::Result<()> {
      Ok(())
    }

    fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
      let mut count = 0;
      for val in buf {
        *val = 1;
        count += 1;
      }
      Ok(count)
    }

    fn write(&self, buf: &[u8]) -> io::Result<usize> {
      let mut count = 0;
      for val in buf {
        unsafe { DATA.push(*val) };
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

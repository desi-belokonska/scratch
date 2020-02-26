pub mod net {
  use crate::util::into_io_error;
  use nix::sys::socket::{
    accept, bind, getpeername, listen, socket, AddressFamily, InetAddr, IpAddr, SockAddr, SockFlag,
    SockProtocol, SockType,
  };
  use nix::unistd::{close, read, write};
  use std::io::{Error, ErrorKind, Read, Result, Write};
  use std::net::{SocketAddr, ToSocketAddrs};

  pub struct Socket(i32);

  pub trait SocketLike {
    fn new() -> Result<Box<Socket>>;
    fn get_peer_name(&self) -> Result<SocketAddr>;
    fn accept(&self) -> Result<Box<Self>>;
    fn bind(&self, addr: SocketAddr) -> Result<()>;
    fn listen(&self, backlog: usize) -> Result<()>;
    fn close(&self) -> Result<()>;
    fn read(&self, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, buf: &[u8]) -> Result<usize>;
  }

  impl Socket {
    fn new() -> Result<Socket> {
      match socket(
        AddressFamily::Inet,
        SockType::Stream,
        SockFlag::empty(),
        SockProtocol::Tcp,
      ) {
        Ok(raw_fd) => Ok(Socket(raw_fd)),
        Err(err) => Err(into_io_error(err)),
      }
    }
  }

  impl SocketLike for Socket {
    fn new() -> Result<Box<Socket>> {
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
    fn get_peer_name(&self) -> Result<SocketAddr> {
      let addr = getpeername(self.0).map_err(into_io_error)?;
      match addr {
        SockAddr::Inet(iaddr) => Ok(iaddr.to_std()),
        _ => Err(Error::from(ErrorKind::Other)),
      }
    }

    fn accept(&self) -> Result<Box<Socket>> {
      match accept(self.0) {
        Ok(raw_fd) => Ok(Box::new(Socket(raw_fd))),
        Err(err) => Err(into_io_error(err)),
      }
    }

    fn bind(&self, addr: SocketAddr) -> Result<()> {
      let address = SockAddr::new_inet(InetAddr::new(IpAddr::from_std(&addr.ip()), addr.port()));
      bind(self.0, &address).map_err(into_io_error)
    }

    fn listen(&self, backlog: usize) -> Result<()> {
      listen(self.0, backlog).map_err(into_io_error)
    }

    fn close(&self) -> Result<()> {
      close(self.0).map_err(into_io_error)
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
      read(self.0, buf).map_err(into_io_error)
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
      write(self.0, buf).map_err(into_io_error)
    }
  }

  pub struct Incoming<'a> {
    listener: &'a TcpListener,
  }

  impl<'a> Iterator for Incoming<'a> {
    type Item = Result<TcpStream>;

    fn next(&mut self) -> Option<Result<TcpStream>> {
      Some(self.listener.accept().map(|p| p.0))
    }
  }

  pub struct TcpStream {
    inner: Socket,
  }

  impl TcpStream {
    pub fn close(&self) -> Result<()> {
      self.inner.close()
    }
  }

  impl Drop for TcpStream {
    fn drop(&mut self) {
      self.inner.close().unwrap()
    }
  }

  impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
      self.inner.read(buf)
    }
  }

  impl Read for &TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
      self.inner.read(buf)
    }
  }

  impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
      self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
      Ok(())
    }
  }

  impl Write for &TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
      self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
      Ok(())
    }
  }

  pub struct TcpListener {
    inner: Socket,
  }

  impl TcpListener {
    pub fn bind(ip: impl ToSocketAddrs) -> Result<TcpListener> {
      let ip_addresses = ip.to_socket_addrs()?;

      for addr in ip_addresses {
        if addr.is_ipv4() {
          let sock = Socket::new()?;
          sock.bind(addr)?;
          sock.listen(128)?;
          return Ok(TcpListener { inner: sock });
        }
      }
      Err(Error::from(ErrorKind::Other))
    }

    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
      let new_socket = self.inner.accept()?;
      let socket_addr = new_socket.get_peer_name()?;
      return Ok((TcpStream { inner: *new_socket }, socket_addr));
    }

    pub fn incoming(&self) -> Incoming {
      Incoming { listener: self }
    }
  }
}

pub mod util {
  use std::io::Error;

  pub fn into_io_error(err: nix::Error) -> Error {
    Error::from(err.as_errno().unwrap())
  }
}

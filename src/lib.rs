use nix::sys::socket::{
  accept, bind, getpeername, listen, socket, AddressFamily, InetAddr, IpAddr, SockAddr, SockFlag,
  SockProtocol, SockType,
};
use nix::unistd::{close, read, write};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

pub struct Socket(i32);

impl Socket {
  fn new() -> io::Result<Socket> {
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

  fn get_peer_name(&self) -> io::Result<SocketAddr> {
    let addr = match getpeername(self.0) {
      Ok(sock_addr) => sock_addr,
      Err(err) => return Err(into_io_error(err)),
    };
    match addr {
      SockAddr::Inet(iaddr) => Ok(iaddr.to_std()),
      _ => Err(io::Error::from(io::ErrorKind::Other)),
    }
  }

  fn accept(&self) -> io::Result<Socket> {
    match accept(self.0) {
      Ok(raw_fd) => Ok(Socket(raw_fd)),
      Err(err) => Err(into_io_error(err)),
    }
  }

  fn bind(&self, addr: SocketAddr) -> io::Result<()> {
    let address = SockAddr::new_inet(InetAddr::new(IpAddr::from_std(&addr.ip()), addr.port()));
    match bind(self.0, &address) {
      Err(err) => Err(into_io_error(err)),
      _ => Ok(()),
    }
  }

  fn listen(&self, backlog: usize) -> io::Result<()> {
    match listen(self.0, backlog) {
      Err(err) => Err(into_io_error(err)),
      _ => Ok(()),
    }
  }

  fn close(&self) -> io::Result<()> {
    match close(self.0) {
      Err(err) => Err(into_io_error(err)),
      _ => Ok(()),
    }
  }

  fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
    match read(self.0, buf) {
      Ok(bytes_read) => Ok(bytes_read),
      Err(err) => Err(into_io_error(err)),
    }
  }

  fn write(&self, buf: &[u8]) -> io::Result<usize> {
    match write(self.0, buf) {
      Ok(bytes_written) => Ok(bytes_written),
      Err(err) => Err(into_io_error(err)),
    }
  }
}

pub struct Incoming<'a> {
  listener: &'a TcpListener,
}

impl<'a> Iterator for Incoming<'a> {
  type Item = io::Result<TcpStream>;
  fn next(&mut self) -> Option<io::Result<TcpStream>> {
    Some(self.listener.accept().map(|p| p.0))
  }
}

pub struct TcpStream {
  inner: Socket,
}

impl TcpStream {
  pub fn close(&self) -> io::Result<()> {
    self.inner.close()
  }
}

impl io::Read for TcpStream {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.inner.read(buf)
  }
}

impl io::Read for &TcpStream {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.inner.read(buf)
  }
}

impl io::Write for TcpStream {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.inner.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl io::Write for &TcpStream {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.inner.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

pub struct TcpListener {
  inner: Socket,
}

impl TcpListener {
  pub fn bind(ip: impl ToSocketAddrs) -> io::Result<TcpListener> {
    let ip_addresses = ip.to_socket_addrs()?;

    for addr in ip_addresses {
      if addr.is_ipv4() {
        let sock = Socket::new()?;
        sock.bind(addr)?;
        sock.listen(128)?;
        return Ok(TcpListener { inner: sock });
      }
    }
    Err(io::Error::from(io::ErrorKind::Other))
  }

  pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
    let new_socket = self.inner.accept()?;
    let socket_addr = new_socket.get_peer_name()?;
    return Ok((TcpStream { inner: new_socket }, socket_addr));
  }

  pub fn incoming(&self) -> Incoming {
    Incoming { listener: self }
  }
}

fn into_io_error(err: nix::Error) -> io::Error {
  io::Error::from(err.as_errno().unwrap())
}

use nix::sys::socket::{
  accept, bind, listen, socket, AddressFamily, InetAddr, IpAddr, SockAddr, SockFlag, SockProtocol,
  SockType,
};
use nix::unistd::{close, read, write};

const PORT: u16 = 8000;

fn main() {
  let hello = "Hello from server";
  let server_fd = socket(
    AddressFamily::Inet,
    SockType::Stream,
    SockFlag::empty(),
    SockProtocol::Tcp,
  )
  .expect("Error in socket");

  let address = SockAddr::new_inet(InetAddr::new(IpAddr::new_v4(127, 0, 0, 1), PORT));

  bind(server_fd, &address).expect("Error in bind");

  listen(server_fd, 10).expect("Error in listen");

  loop {
    let new_socket = accept(server_fd).expect("Error in accept");
    let buffer = &mut [0; 30000];
    read(new_socket, buffer).expect("Error in read");
    println!(
      "{:?}",
      std::str::from_utf8(buffer).expect("Error in str conv")
    );
    write(new_socket, hello.as_bytes()).expect("Error in write");
    println!("-> Hello message sent");
    close(new_socket).expect("Error in close");
  }
}

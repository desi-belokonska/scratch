use pretty_env_logger;
use scratch::net::http::{FileServer, Server};
use std::io;

const PUBLIC: &str = "public";

fn main() -> io::Result<()> {
  pretty_env_logger::init();

  let file_server = FileServer::new(PUBLIC);

  Server::bind("127.0.0.1:8000").serve(file_server)
}

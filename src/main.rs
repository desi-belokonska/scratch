use scratch::net::http::{Request, Response, Server};
use scratch::net::tcp::*;
use std::fs;
use std::io::{BufWriter, Result, Write};

const PUBLIC: &str = "public";
const HELLO: &str = "Hello from server";

fn main() -> Result<()> {
  Server::bind("127.0.0.1:8000").serve(handle_request)
}

fn handle_request(
  request: Request,
  mut response_writer: BufWriter<&TcpStream<Socket>>,
) -> Result<()> {
  let file_path = format!("{}{}", PUBLIC, request.url().path());

  match fs::read(&file_path) {
    Ok(content) => {
      let res: Response = Response::builder()
        .body(&content)
        .header(("Content-Type".to_string(), "text/html".to_string()))
        .into();
      response_writer.write_all(res.to_string().as_bytes())?
    }
    Err(err) => {
      println!("🗂  path:{}; {:?}", file_path, err);
      response_writer.write_all(HELLO.as_bytes())?
    }
  };

  println!("📮 : Hello message sent");
  Ok(())
}

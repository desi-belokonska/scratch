use scratch::net::http::Request;
use scratch::net::tcp::*;
use scratch::net::util;
use std::fs;
use std::io::{BufReader, BufWriter, Read, Result, Write};

const PUBLIC: &str = "public";

fn main() -> Result<()> {
  let hello = "Hello from server";

  let listener = TcpListener::<Socket>::bind("127.0.0.1:8000")?;

  for stream in listener.incoming() {
    let stream = stream?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let buffer = &mut [0; 30000];

    let bytes_read = reader.read(buffer)?;

    let raw_request = util::buffer_to_str(buffer, bytes_read);

    let parsed = Request::parse(raw_request).expect("Wrong request");

    let file_path = format!("{}{}", PUBLIC, parsed.url().path());

    match fs::read(&file_path) {
      Ok(content) => writer.write_all(&content)?,
      Err(err) => {
        println!("🗂  path:{}; {:?}", file_path, err);
        writer.write_all(hello.as_bytes())?
      }
    };

    println!("raw {}", raw_request);
    println!("parsed: {:?}", parsed);

    println!("📮 : Hello message sent");
  }
  Ok(())
}

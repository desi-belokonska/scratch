use crate::net::http::{Request, Response};
use crate::net::{TcpListener, TcpStream};
use crate::thread::*;
use num_cpus;
use std::io;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::net::ToSocketAddrs;
use std::time::SystemTime;

// const DEFAULT_MAX_HEADER_BYTES: u32 = 1 << 20;

pub struct Server {
  inner: TcpListener,
}

impl Server {
  pub fn bind(addr: impl ToSocketAddrs) -> Self {
    let listener =
      TcpListener::bind(addr).unwrap_or_else(|e| panic!("error binding to address: {}", e));
    Server { inner: listener }
  }

  pub fn serve<'a, F>(&self, handle_fn: F) -> io::Result<()>
  where
    F: FnOnce(Request) -> io::Result<Response> + Send + 'static + Copy,
  {
    match self.inner.local_addr() {
      Ok(addr) => println!("Server listening on http://{}", addr),
      Err(err) => error!("Error getting local address: {}", err),
    }

    // Create a thread pool with as many threads as there are logical cpus
    let logical_cpus = num_cpus::get();
    let pool = ThreadPool::new(logical_cpus);

    for stream in self.inner.incoming() {
      let stream = stream?;

      pool.execute(move || -> io::Result<()> {
        time_request(|| -> io::Result<()> {
          let request = read_from_client(&stream)?;
          let response = handle_fn(request)?;
          respond_to_client(&stream, response)?;

          Ok(())
        })
      });
    }
    Ok(())
  }
}

fn read_from_client(stream: &TcpStream) -> io::Result<Request> {
  let mut reader = BufReader::new(stream);
  let buffer = &mut [0; 30000];

  reader.read(buffer).expect("reading failed");

  let raw_request = String::from_utf8_lossy(&buffer[..]);

  let request = Request::parse(&raw_request).map_err(|_| Error::from(ErrorKind::InvalidInput));

  info!("{}", raw_request);

  return request;
}

fn respond_to_client(stream: &TcpStream, response: Response) -> io::Result<()> {
  let mut writer = BufWriter::new(stream);
  writer.write_all(&response.as_bytes())
}

fn time_request(func: impl Fn() -> io::Result<()>) -> io::Result<()> {
  let now = SystemTime::now();
  func()?;
  match now.elapsed() {
    Ok(elapsed) => {
      info!(
        "took: {} microsecs ({} secs)",
        elapsed.as_micros(),
        elapsed.as_secs()
      );
    }
    Err(e) => {
      error!("Error gettting time: {:?}", e);
    }
  };
  Ok(())
}

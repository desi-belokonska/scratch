use crate::net::http::{Handler, Request, Response, Status};
use mime_guess::from_path;
use std::fs;
use std::io;

pub struct FileServer {
  dir: &'static str,
}

impl FileServer {
  pub fn new(dir: &'static str) -> Self {
    FileServer { dir }
  }
}

impl Handler for FileServer {
  fn handle(&self, req: Request) -> io::Result<Response> {
    let file_path = format!("{}{}", self.dir, req.url().path());

    match fs::read(&file_path) {
      Ok(content) => {
        let mime_type_guess = from_path(&file_path).first_raw().unwrap_or("text/plain");
        Ok(
          Response::builder()
            .body(content)
            .header(("Content-Type".to_string(), mime_type_guess.to_string()))
            .into(),
        )
      }
      Err(_) => Ok(Response::builder().status(Status::NotFound).into()),
    }
  }
}

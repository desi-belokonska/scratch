use mime_guess::from_path;
use pretty_env_logger;
use scratch::net::http::{HandlerFunc, Request, Response, Server, Status};
use std::fs;
use std::io;

const PUBLIC: &str = "public";

fn main() -> io::Result<()> {
  pretty_env_logger::init();
  let handler = HandlerFunc::new(handle_request);

  Server::bind("127.0.0.1:8000").serve(handler)
}

fn handle_request<'a>(request: Request) -> io::Result<Response> {
  let file_path = format!("{}{}", PUBLIC, request.url().path());

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

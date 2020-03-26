use mime_guess::from_path;
use pretty_env_logger;
use scratch::net::http::{Request, Response, Server, Status};
use std::fs;
use std::io::Result;

const PUBLIC: &str = "public";

fn main() -> Result<()> {
  pretty_env_logger::init();
  Server::bind("127.0.0.1:8001").serve(handle_request)
}

fn handle_request<'a>(request: Request) -> Result<Response> {
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

use scratch::net::http::{Request, Response, Server, Status};
use std::fs;
use std::io::Result;

const PUBLIC: &str = "public";

fn main() -> Result<()> {
  Server::bind("127.0.0.1:8000").serve(handle_request)
}

fn handle_request<'a>(request: Request) -> Result<Response> {
  let file_path = format!("{}{}", PUBLIC, request.url().path());

  match fs::read(&file_path) {
    Ok(content) => Ok(
      Response::builder()
        .body(content)
        .header(("Content-Type".to_string(), "text/html".to_string()))
        .into(),
    ),
    Err(_) => Ok(Response::builder().status(Status::NotFound).into()),
  }
}

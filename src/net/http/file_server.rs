use crate::net::http::{Handler, Request, Response, Status};
use handlebars::Handlebars;
use mime_guess::from_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Dir<'a> {
  base_dir: &'a str,
  dir_name: &'a str,
  child_dirs: Vec<PathBuf>,
  files: Vec<PathBuf>,
}

impl<'a> Dir<'a> {
  // one possible implementation of walking a directory only visiting files
  fn new(base_dir: &'a str, dir: &'a Path) -> io::Result<Dir<'a>> {
    let mut result = Dir {
      base_dir,
      dir_name: dir
        .to_str()
        .ok_or(io::Error::from(io::ErrorKind::NotFound))?,
      child_dirs: Vec::new(),
      files: Vec::new(),
    };

    if dir.is_dir() {
      for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
          result.child_dirs.push(
            path
              .strip_prefix(base_dir)
              .expect("can't strip")
              .to_path_buf(),
          );
        } else {
          result.files.push(
            entry
              .path()
              .strip_prefix(base_dir)
              .expect("can't strip")
              .to_path_buf(),
          );
        }
      }
    }
    Ok(result)
  }
}

pub struct FileServer {
  dir: &'static str,
}

impl FileServer {
  pub fn new(dir: &'static str) -> Self {
    FileServer { dir }
  }

  fn create_template<'a>(&self, dir: &'a Path) -> String {
    let mut handlebars = Handlebars::new();
    // register the template. The template string will be verified and compiled.
    let source = "<h1>{{dir_name}}</h1>\
                  {{#each child_dirs}}
                  <a href=\"{{this}}\">{{this}}</a>\
                  <br>\
                  {{/each}}
                  {{#each files}}
                  <a href=\"{{this}}\">{{this}}</a>\
                  <br>\
                  {{/each}}";
    assert!(handlebars
      .register_template_string("dir_template", source)
      .is_ok());

    let dir = Dir::new(self.dir, dir).expect("error creating dir");
    handlebars
      .render("dir_template", &dir)
      .expect("can't render template")
  }
}

impl Handler for FileServer {
  fn handle(&self, req: Request) -> io::Result<Response> {
    let mut file_path = PathBuf::from(self.dir);
    trace!("1 - {:?}", file_path);

    file_path.push(
      req
        .url()
        .path()
        .get(1..)
        .expect("doesn't have leading slash?"),
    );

    trace!("2 - {:?}", file_path);

    if file_path.is_dir() {
      let template = self.create_template(&file_path);
      Ok(
        Response::builder()
          .body(template.into_bytes())
          .header(("Content-Type".to_string(), String::from("text/html")))
          .into(),
      )
    } else {
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
}

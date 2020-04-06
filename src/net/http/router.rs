use super::common::Url;
use super::request::Method;
use super::{Request, Response, Status};
use std::collections::HashMap;
use std::io;

pub trait Handler: Send + Sync + 'static {
  fn handle(&self, req: Request) -> io::Result<Response>;
}

pub struct HandlerFunc {
  func: Box<dyn Fn(Request) -> io::Result<Response> + Send + Sync + 'static>,
}

impl HandlerFunc {
  pub fn new(func: impl Fn(Request) -> io::Result<Response> + Send + Sync + 'static) -> Self {
    HandlerFunc {
      func: Box::new(func),
    }
  }
}

impl Handler for HandlerFunc {
  fn handle(&self, req: Request) -> io::Result<Response> {
    (self.func)(req)
  }
}

impl<F> Handler for F
where
  F: Send + Sync + 'static + Fn(Request) -> io::Result<Response>,
{
  fn handle(&self, req: Request) -> io::Result<Response> {
    (*self)(req)
  }
}

impl Handler for Box<dyn Handler> {
  fn handle(&self, req: Request) -> io::Result<Response> {
    (**self).handle(req)
  }
}

pub struct Router {
  // The routers, specialized by url.
  router_map: HashMap<Url, HashMap<Method, Box<dyn Handler>>>,
}

impl Default for Router {
  fn default() -> Self {
    Self::new()
  }
}

impl Router {
  pub fn new() -> Self {
    Router {
      router_map: HashMap::new(),
    }
  }

  pub fn route(&mut self, method: Method, url: Url, handler: impl Handler) {
    self
      .router_map
      .entry(url)
      .or_insert_with(HashMap::new)
      .insert(method, Box::new(handler));
  }

  pub fn get(&mut self, url: Url, handler: impl Handler) {
    self.route(Method::GET, url, handler)
  }

  pub fn post(&mut self, url: Url, handler: impl Handler) {
    self.route(Method::POST, url, handler)
  }

  pub fn put(&mut self, url: Url, handler: impl Handler) {
    self.route(Method::PUT, url, handler)
  }

  pub fn delete(&mut self, url: Url, handler: impl Handler) {
    self.route(Method::DELETE, url, handler)
  }
}

impl Handler for Router {
  fn handle(&self, req: Request) -> io::Result<Response> {
    let handler = self
      .router_map
      .get(req.url())
      .and_then(|method_map| method_map.get(req.method()));

    match handler {
      Some(handler) => handler.handle(req),
      None => Ok(Response::builder().status(Status::NotFound).into()),
    }
  }
}

// pub struct Recognizer {
//   method_map: HashMap<Method, Box<dyn Handler>>,
// }

// impl Recognizer {
//   pub fn get_inner(&mut self) -> &mut HashMap<Method, Box<dyn Handler>> {
//     self.method_map
//   }
// }

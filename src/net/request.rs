use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind, Result as IoResult};
use std::str::FromStr;

#[derive(Debug)]
pub struct ParseError();

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Unable to parse input")
  }
}

pub trait Parse {
  fn parse(txt: &str) -> Result<(&str, Self), ParseError>
  where
    Self: std::marker::Sized;
}

#[derive(Default, Debug)]
pub struct Request {
  method: Method,
  url: Url,
  version: Version,
  headers: Headers,
  body: String,
}

impl Request {
  pub fn parse(raw: &str) -> Result<Self, ParseError> {
    let (rest, method) = Method::parse(raw)?;
    let (rest, url) = Url::parse(rest)?;
    let (rest, version) = Version::parse(rest)?;
    let (body, headers) = Headers::parse(rest)?;
    let req: Request = Request::builder()
      .method(method)
      .url(url)
      .version(version)
      .headers(headers)
      .body(body.to_string())
      .into();
    Ok(req)
  }

  pub fn builder() -> RequestBuilder {
    RequestBuilder(Default::default())
  }

  pub fn method(&self) -> &Method {
    &self.method
  }

  pub fn headers(&self) -> &Headers {
    &self.headers
  }

  pub fn url(&self) -> &Url {
    &self.url
  }

  pub fn version(&self) -> &Version {
    &self.version
  }

  pub fn body(&self) -> &str {
    &self.body
  }
}

impl From<RequestBuilder> for Request {
  fn from(builder: RequestBuilder) -> Self {
    builder.0
  }
}

pub struct RequestBuilder(Request);

impl RequestBuilder {
  pub fn method(mut self, meth: Method) -> Self {
    self.0.method = meth;
    self
  }

  pub fn headers(mut self, headers: Headers) -> Self {
    self.0.headers = headers;
    self
  }

  pub fn url(mut self, url: Url) -> Self {
    self.0.url = url;
    self
  }

  pub fn version(mut self, version: Version) -> Self {
    self.0.version = version;
    self
  }

  pub fn body(mut self, body: String) -> Self {
    self.0.body = body;
    self
  }
}

#[derive(Default, Debug)]
pub struct Headers(HashMap<String, String>);
type Header = (String, String);

impl Headers {
  fn parse_one(txt: &str) -> Result<(&str, Header), ParseError> {
    let mut itr = txt.splitn(2, "\r\n");

    let header_line = itr.next().ok_or(ParseError())?;
    let rest = itr.next().ok_or(ParseError())?;

    let mut kv = header_line.splitn(2, ": ");

    let field_name = kv.next().ok_or(ParseError())?.to_string();
    let field_value = kv.next().ok_or(ParseError())?.to_string();

    Ok((rest, (field_name, field_value)))
  }
}

impl Parse for Headers {
  fn parse(txt: &str) -> Result<(&str, Self), ParseError> {
    let mut headers = HashMap::new();
    let mut rest = txt;
    loop {
      if rest.starts_with("\r\n") {
        let body = rest.splitn(2, "\r\n").nth(1).ok_or(ParseError())?;
        return Ok((body, Headers(headers)));
      }
      let (txt_rest, (name, value)) = Headers::parse_one(rest)?;
      headers.insert(name, value);
      rest = txt_rest;
    }
  }
}

#[derive(Default, Debug, PartialEq)]
pub struct Url {
  path: String,
}

impl Url {
  pub fn path(&self) -> &str {
    &self.path
  }
}

impl Parse for Url {
  fn parse(txt: &str) -> Result<(&str, Self), ParseError> {
    let mut itr = txt.splitn(2, " ");

    let url = itr.next().ok_or(ParseError())?;
    let rest = itr.next().ok_or(ParseError())?;

    Ok((
      rest,
      Url {
        path: url.to_string(),
      },
    ))
  }
}

#[derive(Default, Debug)]
pub struct Version {
  major: u8,
  minor: u8,
}

impl Parse for Version {
  fn parse(txt: &str) -> Result<(&str, Self), ParseError> {
    let mut itr = txt.splitn(2, "\r\n");

    let version = itr.next().ok_or(ParseError())?;
    let rest = itr.next().ok_or(ParseError())?;

    let mut version_iter = version
      .splitn(2, "HTTP/")
      .skip(1)
      .next()
      .ok_or(ParseError())?
      .split(".");

    let major = version_iter
      .next()
      .ok_or(ParseError())?
      .parse::<u8>()
      .map_err(|_| ParseError())?;

    let minor = version_iter
      .next()
      .ok_or(ParseError())?
      .parse::<u8>()
      .map_err(|_| ParseError())?;

    Ok((rest, Version { major, minor }))
  }
}

#[derive(Debug, PartialEq)]
pub enum Method {
  OPTIONS,
  GET,
  HEAD,
  POST,
  PUT,
  DELETE,
  TRACE,
  CONNECT,
}

impl FromStr for Method {
  type Err = Error;

  fn from_str(string: &str) -> IoResult<Self> {
    if string == "OPTIONS" {
      Ok(Method::OPTIONS)
    } else if string == "GET" {
      Ok(Method::GET)
    } else if string == "HEAD" {
      Ok(Method::HEAD)
    } else if string == "POST" {
      Ok(Method::POST)
    } else if string == "PUT" {
      Ok(Method::PUT)
    } else if string == "DELETE" {
      Ok(Method::DELETE)
    } else if string == "TRACE" {
      Ok(Method::TRACE)
    } else {
      Err(Error::from(ErrorKind::InvalidInput))
    }
  }
}

impl Default for Method {
  fn default() -> Self {
    return Method::GET;
  }
}

impl Parse for Method {
  fn parse(txt: &str) -> Result<(&str, Self), ParseError> {
    let mut itr = txt.splitn(2, " ");

    let method = itr.next().ok_or(ParseError())?;
    let rest = itr.next().ok_or(ParseError())?;

    match method.parse() {
      Ok(meth) => Ok((rest, meth)),
      _ => Err(ParseError()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const SAMPLE_REQUEST: &str = "GET /hello.htm HTTP/1.1\r\nUser-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nHost: www.tutorialspoint.com\r\nAccept-Language: en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n";

  const BAD_REQUEST: &str = "GET /hello.htm HTTP/1.1\r\nUser-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nHost: www.tutorialspoint.com\r\nAccept-Language en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n";

  const SAMPLE_REQUEST_URL : &str = "/hello.htm HTTP/1.1\r\nUser-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nHost: www.tutorialspoint.com\r\nAccept-Language: en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n";

  const SAMPLE_REQUEST_VERSION : &str = "HTTP/1.1\r\nUser-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nHost: www.tutorialspoint.com\r\nAccept-Language: en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n";

  const SAMPLE_REQUEST_HEADERS : &str = "User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\nHost: www.tutorialspoint.com\r\nAccept-Language: en-us\r\nAccept-Encoding: gzip, deflate\r\nConnection: Keep-Alive\r\n\r\n";

  #[test]
  fn good_parse() {
    assert!(Request::parse(SAMPLE_REQUEST).is_ok());
  }

  #[test]
  fn bad_parse() {
    assert!(Request::parse(BAD_REQUEST).is_err());
  }

  #[test]
  fn parse_method() {
    let parsed = Method::parse(SAMPLE_REQUEST);
    // println!("parse_method: {:?}", parsed);
    assert!(parsed.is_ok() && parsed.unwrap().0 == SAMPLE_REQUEST_URL);
  }

  #[test]
  fn parse_url() {
    let parsed = Url::parse(SAMPLE_REQUEST_URL);
    // println!("parse_url: {:?}", parsed);
    assert!(parsed.is_ok() && parsed.unwrap().0 == SAMPLE_REQUEST_VERSION);
  }

  #[test]
  fn parse_version() {
    let parsed = Version::parse(SAMPLE_REQUEST_VERSION);
    // println!("parse_version: {:?}", parsed);
    assert!(parsed.is_ok());
  }

  #[test]
  fn parse_headers() {
    let parsed = Headers::parse(SAMPLE_REQUEST_HEADERS);
    // println!("parse_headers: {:?}", parsed);
    assert!(parsed.is_ok() && parsed.unwrap().0 == "");
  }
}

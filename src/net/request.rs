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

#[derive(Debug)]
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

impl Default for Version {
  fn default() -> Self {
    Version { major: 1, minor: 1 }
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

#[derive(Default, Debug)]
pub struct Response {
  status: Status, //code and text
  version: Version,
  url: Url,         // can use same
  headers: Headers, //can use same
  body: String,
}

impl Response {
  pub fn builder() -> ResponseBuilder {
    ResponseBuilder(Default::default())
  }

  pub fn status(&self) -> &Status {
    &self.status
  }

  pub fn version(&self) -> &Version {
    &self.version
  }

  pub fn url(&self) -> &Url {
    &self.url
  }

  pub fn headers(&self) -> &Headers {
    &self.headers
  }

  pub fn body(&self) -> &str {
    &self.body
  }
}

impl From<ResponseBuilder> for Response {
  fn from(builder: ResponseBuilder) -> Self {
    builder.0
  }
}

pub struct ResponseBuilder(Response);

impl ResponseBuilder {
  pub fn status(mut self, status: Status) -> Self {
    self.0.status = status;
    self
  }

  pub fn version(mut self, version: Version) -> Self {
    self.0.version = version;
    self
  }

  pub fn url(mut self, url: Url) -> Self {
    self.0.url = url;
    self
  }

  pub fn headers(mut self, headers: Headers) -> Self {
    self.0.headers = headers;
    self
  }

  pub fn body(mut self, body: String) -> Self {
    self.0.body = body;
    self
  }
}

#[derive(Debug)]
pub enum Status {
  Continue = 100,           // RFC 7231, 6.2.1
  SwitchingProtocols = 101, // RFC 7231, 6.2.2
  Processing = 102,         // RFC 2518, 10.1
  EarlyHints = 103,         // RFC 8297

  OK = 200,                   // RFC 7231, 6.3.1
  Created = 201,              // RFC 7231, 6.3.2
  Accepted = 202,             // RFC 7231, 6.3.3
  NonAuthoritativeInfo = 203, // RFC 7231, 6.3.4
  NoContent = 204,            // RFC 7231, 6.3.5
  ResetContent = 205,         // RFC 7231, 6.3.6
  PartialContent = 206,       // RFC 7233, 4.1
  MultiStatus = 207,          // RFC 4918, 11.1
  AlreadyReported = 208,      // RFC 5842, 7.1
  IMUsed = 226,               // RFC 3229, 10.4.1

  MultipleChoices = 300,  // RFC 7231, 6.4.1
  MovedPermanently = 301, // RFC 7231, 6.4.2
  Found = 302,            // RFC 7231, 6.4.3
  SeeOther = 303,         // RFC 7231, 6.4.4
  NotModified = 304,      // RFC 7232, 4.1
  UseProxy = 305,         // RFC 7231, 6.4.5

  TemporaryRedirect = 307, // RFC 7231, 6.4.7
  PermanentRedirect = 308, // RFC 7538, 3

  BadRequest = 400,                   // RFC 7231, 6.5.1
  Unauthorized = 401,                 // RFC 7235, 3.1
  PaymentRequired = 402,              // RFC 7231, 6.5.2
  Forbidden = 403,                    // RFC 7231, 6.5.3
  NotFound = 404,                     // RFC 7231, 6.5.4
  MethodNotAllowed = 405,             // RFC 7231, 6.5.5
  NotAcceptable = 406,                // RFC 7231, 6.5.6
  ProxyAuthRequired = 407,            // RFC 7235, 3.2
  RequestTimeout = 408,               // RFC 7231, 6.5.7
  Conflict = 409,                     // RFC 7231, 6.5.8
  Gone = 410,                         // RFC 7231, 6.5.9
  LengthRequired = 411,               // RFC 7231, 6.5.10
  PreconditionFailed = 412,           // RFC 7232, 4.2
  RequestEntityTooLarge = 413,        // RFC 7231, 6.5.11
  RequestURITooLong = 414,            // RFC 7231, 6.5.12
  UnsupportedMediaType = 415,         // RFC 7231, 6.5.13
  RequestedRangeNotSatisfiable = 416, // RFC 7233, 4.4
  ExpectationFailed = 417,            // RFC 7231, 6.5.14
  Teapot = 418,                       // RFC 7168, 2.3.3
  MisdirectedRequest = 421,           // RFC 7540, 9.1.2
  UnprocessableEntity = 422,          // RFC 4918, 11.2
  Locked = 423,                       // RFC 4918, 11.3
  FailedDependency = 424,             // RFC 4918, 11.4
  TooEarly = 425,                     // RFC 8470, 5.2.
  UpgradeRequired = 426,              // RFC 7231, 6.5.15
  PreconditionRequired = 428,         // RFC 6585, 3
  TooManyRequests = 429,              // RFC 6585, 4
  RequestHeaderFieldsTooLarge = 431,  // RFC 6585, 5
  UnavailableForLegalReasons = 451,   // RFC 7725, 3

  InternalServerError = 500,           // RFC 7231, 6.6.1
  NotImplemented = 501,                // RFC 7231, 6.6.2
  BadGateway = 502,                    // RFC 7231, 6.6.3
  ServiceUnavailable = 503,            // RFC 7231, 6.6.4
  GatewayTimeout = 504,                // RFC 7231, 6.6.5
  HTTPVersionNotSupported = 505,       // RFC 7231, 6.6.6
  VariantAlsoNegotiates = 506,         // RFC 2295, 8.1
  InsufficientStorage = 507,           // RFC 4918, 11.5
  LoopDetected = 508,                  // RFC 5842, 7.2
  NotExtended = 510,                   // RFC 2774, 7
  NetworkAuthenticationRequired = 511, // RFC 6585, 6
}

impl Default for Status {
  fn default() -> Self {
    Status::OK
  }
}

impl Status {
  pub fn code(&self) -> u8 {
    self as *const Status as u8
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

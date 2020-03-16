use super::util::buffer_to_str;
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

impl<'a> fmt::Display for Version {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "HTTP/{}.{}", self.major, self.minor)
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
pub struct Response<'a> {
  status: Status, //code and text
  version: Version,
  url: Url,         // can use same
  headers: Headers, //can use same
  body: &'a [u8],
}

impl<'a> Response<'a> {
  pub fn builder() -> ResponseBuilder<'a> {
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

  pub fn body(&self) -> &[u8] {
    &self.body
  }
}

impl<'a> From<ResponseBuilder<'a>> for Response<'a> {
  fn from(builder: ResponseBuilder<'a>) -> Self {
    builder.0
  }
}

impl<'a> fmt::Display for Response<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut result = format!("{} {}\n", self.version(), self.status());
    for (header, field) in self.headers().0.iter() {
      result = format!("{}{}: {}\n", result, header, field);
    }
    result = format!("{}\n", result);
    if self.body().len() > 0 {
      result = format!(
        "{}{}",
        result,
        buffer_to_str(self.body(), self.body().len())
      );
    }
    write!(f, "{}", result)
  }
}

pub struct ResponseBuilder<'a>(Response<'a>);

impl<'a> ResponseBuilder<'a> {
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

  pub fn header(mut self, (key, value): (String, String)) -> Self {
    self.0.headers.0.insert(key, value);
    self
  }

  pub fn headers(mut self, headers: Headers) -> Self {
    self.0.headers = headers;
    self
  }

  pub fn body(mut self, body: &'a [u8]) -> Self {
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

impl<'a> fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}", self.code(), self.text())
  }
}

impl Status {
  pub fn code(&self) -> u16 {
    match self {
      Status::Continue => 100,
      Status::SwitchingProtocols => 101,
      Status::Processing => 102,
      Status::EarlyHints => 103,

      Status::OK => 200,
      Status::Created => 201,
      Status::Accepted => 202,
      Status::NonAuthoritativeInfo => 203,
      Status::NoContent => 204,
      Status::ResetContent => 205,
      Status::PartialContent => 206,
      Status::MultiStatus => 207,
      Status::AlreadyReported => 208,
      Status::IMUsed => 226,

      Status::MultipleChoices => 300,
      Status::MovedPermanently => 301,
      Status::Found => 302,
      Status::SeeOther => 303,
      Status::NotModified => 304,
      Status::UseProxy => 305,

      Status::TemporaryRedirect => 307,
      Status::PermanentRedirect => 308,

      Status::BadRequest => 400,
      Status::Unauthorized => 401,
      Status::PaymentRequired => 402,
      Status::Forbidden => 403,
      Status::NotFound => 404,
      Status::MethodNotAllowed => 405,
      Status::NotAcceptable => 406,
      Status::ProxyAuthRequired => 407,
      Status::RequestTimeout => 408,
      Status::Conflict => 409,
      Status::Gone => 410,
      Status::LengthRequired => 411,
      Status::PreconditionFailed => 412,
      Status::RequestEntityTooLarge => 413,
      Status::RequestURITooLong => 414,
      Status::UnsupportedMediaType => 415,
      Status::RequestedRangeNotSatisfiable => 416,
      Status::ExpectationFailed => 417,
      Status::Teapot => 418,
      Status::MisdirectedRequest => 421,
      Status::UnprocessableEntity => 422,
      Status::Locked => 423,
      Status::FailedDependency => 424,
      Status::TooEarly => 425,
      Status::UpgradeRequired => 426,
      Status::PreconditionRequired => 428,
      Status::TooManyRequests => 429,
      Status::RequestHeaderFieldsTooLarge => 431,
      Status::UnavailableForLegalReasons => 451,

      Status::InternalServerError => 500,
      Status::NotImplemented => 501,
      Status::BadGateway => 502,
      Status::ServiceUnavailable => 503,
      Status::GatewayTimeout => 504,
      Status::HTTPVersionNotSupported => 505,
      Status::VariantAlsoNegotiates => 506,
      Status::InsufficientStorage => 507,
      Status::LoopDetected => 508,
      Status::NotExtended => 510,
      Status::NetworkAuthenticationRequired => 511,
    }
  }

  pub fn text(&self) -> &'static str {
    match self {
      Status::Continue => "Continue",
      Status::SwitchingProtocols => "Switching Protocols",
      Status::Processing => "Processing",
      Status::EarlyHints => "Early Hints",

      Status::OK => "OK",
      Status::Created => "Created",
      Status::Accepted => "Accepted",
      Status::NonAuthoritativeInfo => "Non-Authoritative Information",
      Status::NoContent => "No Content",
      Status::ResetContent => "Reset Content",
      Status::PartialContent => "Partial Content",
      Status::MultiStatus => "Multi-Status",
      Status::AlreadyReported => "Already Reported",
      Status::IMUsed => "IM Used",

      Status::MultipleChoices => "Multiple Choices",
      Status::MovedPermanently => "Moved Permanently",
      Status::Found => "Found",
      Status::SeeOther => "See Other",
      Status::NotModified => "Not Modified",
      Status::UseProxy => "Use Proxy",

      Status::TemporaryRedirect => "Temporary Redirect",
      Status::PermanentRedirect => "Permanent Redirect",

      Status::BadRequest => "Bad Request",
      Status::Unauthorized => "Unauthorized",
      Status::PaymentRequired => "Payment Required",
      Status::Forbidden => "Forbidden",
      Status::NotFound => "Not Found",
      Status::MethodNotAllowed => "Method Not Allowed",
      Status::NotAcceptable => "Not Acceptable",
      Status::ProxyAuthRequired => "Proxy Authentication Required",
      Status::RequestTimeout => "Request Timeout",
      Status::Conflict => "Conflict",
      Status::Gone => "Gone",
      Status::LengthRequired => "Length Required",
      Status::PreconditionFailed => "Precondition Failed",
      Status::RequestEntityTooLarge => "Payload Too Large",
      Status::RequestURITooLong => "URI Too Long",
      Status::UnsupportedMediaType => "Unsupported Media Type",
      Status::RequestedRangeNotSatisfiable => "Range Not Satisfiable",
      Status::ExpectationFailed => "Expectation Failed",
      Status::Teapot => "I'm a teapot",
      Status::MisdirectedRequest => "Misdirected Request",
      Status::UnprocessableEntity => "Unprocessable Entity",
      Status::Locked => "Locked",
      Status::FailedDependency => "Failed Dependency",
      Status::TooEarly => "Too Early",
      Status::UpgradeRequired => "Upgrade Required",
      Status::PreconditionRequired => "Precondition Required",
      Status::TooManyRequests => "Too Many Requests",
      Status::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
      Status::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

      Status::InternalServerError => "Internal Server Error",
      Status::NotImplemented => "Not Implemented",
      Status::BadGateway => "Bad Gateway",
      Status::ServiceUnavailable => "Service Unavailable",
      Status::GatewayTimeout => "Gateway Timeout",
      Status::HTTPVersionNotSupported => "HTTP Version Not Supported",
      Status::VariantAlsoNegotiates => "Variant Also Negotiates",
      Status::InsufficientStorage => "Insufficient Storage",
      Status::LoopDetected => "Loop Detected",
      Status::NotExtended => "Not Extended",
      Status::NetworkAuthenticationRequired => "Network Authentication Required",
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

  #[test]
  fn status_code() {
    let stat = Status::OK;
    assert_eq!(stat.code(), 200)
  }
}

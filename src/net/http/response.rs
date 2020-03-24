use super::common::*;
use crate::net::util::buffer_to_str;
use std::fmt;

#[derive(Default, Debug)]
pub struct Response {
  status: Status,
  version: Version,
  url: Url,
  headers: Headers,
  body: Vec<u8>,
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

  pub fn body(&self) -> &[u8] {
    &self.body
  }

  pub fn as_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::from(self.to_string().as_bytes());
    let mut body_bytes = Vec::from(self.body());
    bytes.append(&mut body_bytes);
    bytes
  }
}

impl From<ResponseBuilder> for Response {
  fn from(builder: ResponseBuilder) -> Self {
    builder.0
  }
}

impl fmt::Display for Response {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut result = format!("{} {}\n", self.version(), self.status());
    for (header, field) in self.headers().map.iter() {
      result = format!("{}{}: {}\n", result, header, field);
    }
    result = format!("{}\n", result);
    if self.body().len() > 0 {
      match buffer_to_str(self.body(), self.body().len()) {
        Some(body) => {
          result = format!("{}{}", result, body);
        }
        None => (),
      }
    }
    write!(f, "{}", result)
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

  pub fn header(mut self, (key, value): (String, String)) -> Self {
    self.0.headers.map.insert(key, value);
    self
  }

  pub fn headers(mut self, headers: Headers) -> Self {
    self.0.headers = headers;
    self
  }

  pub fn body(mut self, body: Vec<u8>) -> Self {
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

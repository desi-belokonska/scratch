use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

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
pub struct Headers {
  pub map: HashMap<String, String>,
}

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
        return Ok((body, Headers { map: headers }));
      }
      let (txt_rest, (name, value)) = Headers::parse_one(rest)?;
      headers.insert(name, value);
      rest = txt_rest;
    }
  }
}

#[derive(Default, Debug, PartialEq, Eq, Hash)]
pub struct Url {
  path: String,
}

impl Url {
  pub fn with_path(path: &str) -> Url {
    Url {
      path: path.to_string(),
    }
  }

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

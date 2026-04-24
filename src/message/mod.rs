pub mod method;
pub mod request;
pub mod request_uri;
pub mod response;
pub mod status_code;
pub mod version;

use {
  anyhow::anyhow,
  std::{collections::HashMap, fmt::Write},
  typed_builder::TypedBuilder,
};

const CRLF: &str = "\r\n";

// Represents a request from client to server / response from server to client.
#[derive(Debug, PartialEq, Eq, TypedBuilder)]
pub struct HTTPMessage<'http_message, S>
where
  S: HTTPEncodable<'http_message>,
{
  pub start_line: S,

  #[builder(default = HashMap::new())]
  pub headers: HashMap<&'http_message str, &'http_message str>,

  #[builder(default)]
  pub body: Option<String>,
}

pub trait HTTPEncodable<'http_message>
where
  Self: Sized,
{
  fn http_encode(&self, encoding: &mut String) -> anyhow::Result<()>;
  fn http_decode(encoding: &'http_message str) -> anyhow::Result<Self>;
}

impl<'http_message, S> HTTPEncodable<'http_message> for HTTPMessage<'http_message, S>
where
  S: HTTPEncodable<'http_message>,
{
  // Encodes the HTTP message to a string.
  // The string is then returned.
  fn http_encode(&self, encoding: &mut String) -> anyhow::Result<()> {
    // Encode the start-line section.
    self.start_line.http_encode(encoding)?;
    encoding.push_str(CRLF);

    // Encode the headers section.
    self
      .headers
      .iter()
      .try_for_each(|(key, value)| write!(encoding, "{key}: {value}{CRLF}"))?;
    if let Some(ref body) = self.body {
      write!(encoding, "Content-Length: {}{CRLF}", body.len())?;
    }
    encoding.push_str(CRLF);

    // Encode the request-body section.
    if let Some(ref body) = self.body {
      encoding.push_str(body);
    }

    Ok(())
  }

  fn http_decode(encoding: &'http_message str) -> anyhow::Result<Self> {
    let mut parts = encoding.split(CRLF);

    // Decoding the start-line section.
    let start_line = parts
      .next()
      .ok_or_else(|| anyhow!("Start-line section not found"))
      .and_then(|start_line| S::http_decode(start_line))?;

    // Decoding the headers section.
    let mut headers = HashMap::new();
    while let Some(header) = parts.next().take_if(|part| !part.is_empty()) {
      let mut parts = header.split(": ");

      let key = parts
        .next()
        .ok_or_else(|| anyhow!("header key not found"))?;

      let value = parts
        .next()
        .ok_or_else(|| anyhow!("header value not found"))?;

      headers.insert(key, value);
    }
    parts
      .next()
      .ok_or_else(|| anyhow!("header section ended unexpectedly"))?;

    // Decoding the request body.
    let body = parts.next().map(str::to_string);

    Ok(Self {
      start_line,
      headers,
      body,
    })
  }
}

#[cfg(test)]
mod test {
  use {
    super::*,
    crate::message::{method::HTTPMethod, request::RequestLine, response::StatusLine},
  };

  #[test]
  fn decode_http_request() {
    let encoded_http_request = "GET / HTTP/1.1\r\n\r\n";

    let http_request = HTTPMessage::http_decode(encoded_http_request).unwrap();

    let expected_http_request = HTTPMessage::builder()
      .start_line(RequestLine::builder().method(HTTPMethod::GET).build())
      .build();
    assert_eq!(http_request, expected_http_request);
  }

  #[test]
  fn encode_http_response() {
    let http_response = HTTPMessage::builder()
      .start_line(StatusLine::builder().build())
      .build();

    let mut http_response_encoding = String::new();
    http_response
      .http_encode(&mut http_response_encoding)
      .unwrap();

    assert_eq!(http_response_encoding, "HTTP/1.1 200 OK\r\n\r\n");
  }
}

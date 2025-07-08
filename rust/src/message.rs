use {
  crate::{
    start_line::{RequestLine, StatusLine},
    status_code::HTTPStatusCode,
  },
  anyhow::anyhow,
  std::{collections::HashMap, fmt::Write},
  typed_builder::TypedBuilder,
};

const CRLF: &str = "\r\n";

pub type HTTPRequest<'a> = HTTPMessage<'a, RequestLine<'a>>;
pub type HTTPResponse<'a> = HTTPMessage<'a, StatusLine>;

// Represents requests from client to server / responses from server to client.
#[derive(Debug, PartialEq, Eq, TypedBuilder)]
pub struct HTTPMessage<'a, T>
where
  T: HTTPEncodable<'a>,
{
  pub startLine: T,

  #[builder(default = HashMap::new())]
  pub headers: HashMap<&'a str, &'a str>,

  #[builder(default)]
  pub body: Option<&'a str>,
}

pub trait HTTPEncodable<'a>
where
  Self: Sized,
{
  fn httpEncode(&self, encoding: &mut String) -> anyhow::Result<()>;
  fn httpDecode(encoding: &'a str) -> anyhow::Result<Self>;
}

impl<'a, T> HTTPEncodable<'a> for HTTPMessage<'a, T>
where
  T: HTTPEncodable<'a>,
{
  // Encodes the HTTP message to a string.
  // The string is then returned.
  fn httpEncode(&self, encoding: &mut String) -> anyhow::Result<()> {
    // Encode the start-line section.
    self.startLine.httpEncode(encoding)?;
    encoding.push_str(CRLF);

    // Encode the headers section.
    self
      .headers
      .iter()
      .try_for_each(|(key, value)| write!(encoding, "{key}: {value}{CRLF}"))?;
    encoding.push_str(CRLF);

    // Encode the request-body section.
    if let Some(body) = self.body {
      encoding.push_str(body);
    }

    Ok(())
  }

  fn httpDecode(encoding: &'a str) -> anyhow::Result<Self> {
    let mut parts = encoding.split("\r\n");

    // Decoding the start-line section.
    let startLine = parts
      .next()
      .ok_or_else(|| anyhow!("Start-line section not found"))
      .and_then(|startLine| T::httpDecode(startLine))?;

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
    let body = parts.next();

    Ok(Self {
      startLine,
      headers,
      body,
    })
  }
}

impl<'a, T> HTTPMessage<'a, T>
where
  T: HTTPEncodable<'a>,
{
  pub fn setBody(&mut self, body: &'a str) {
    self.body = Some(body);

    // TODO : Use the stack instead completely (if possible).
    let bodyLen: &'a str = body.len().to_string().leak();

    // Set appropriate HTTP heders.
    self.headers.insert("Content-Type", "text/plain");
    self.headers.insert("Content-Length", bodyLen);
  }
}

impl HTTPResponse<'_> {
  pub fn setStatusCode(&mut self, statusCode: HTTPStatusCode) {
    self.startLine.statusCode = statusCode;
  }
}

#[cfg(test)]
mod test {
  use {super::*, crate::method::HTTPMethod};

  #[test]
  fn decodeHTTPRequest() {
    let encodedHTTPRequest = "GET / HTTP/1.1\r\n\r\n";

    let httpRequest = HTTPMessage::httpDecode(encodedHTTPRequest).unwrap();

    let expectedHTTPRequest = HTTPMessage::builder()
      .startLine(RequestLine::builder().method(HTTPMethod::GET).build())
      .build();
    assert_eq!(httpRequest, expectedHTTPRequest);
  }

  #[test]
  fn encodeHTTPResponse() {
    let httpResponse = HTTPMessage::builder()
      .startLine(StatusLine::builder().build())
      .build();

    let mut httpResponseEncoding = String::new();
    httpResponse.httpEncode(&mut httpResponseEncoding).unwrap();

    assert_eq!(httpResponseEncoding, "HTTP/1.1 200 OK\r\n\r\n");
  }
}

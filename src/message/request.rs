use {
  crate::{
    message::{
      method::HTTPMethod, request_uri::HTTPRequestURI, version::HTTPVersion, HTTPEncodable,
      HTTPMessage,
    },
    utils::ToStr,
  },
  anyhow::anyhow,
  std::fmt::Write,
  typed_builder::TypedBuilder,
};

pub type HTTPRequest<'a> = HTTPMessage<'a, RequestLine<'a>>;

#[derive(Debug, PartialEq, Eq, TypedBuilder)]
pub struct RequestLine<'r> {
  /*
    Indicates the method to be performed on the resource identified by the Request-URI.

    The methods GET and HEAD must be supported by all general-purpose servers. All other methods
    are optional.
  */
  pub method: HTTPMethod,

  // A Uniform Resource Identifier (URI) that identifies the resource upon which to apply the
  // request.
  #[builder(default = HTTPRequestURI::AbsolutePath("/"))]
  pub request_uri: HTTPRequestURI<'r>,

  #[builder(default)]
  pub protocol_version: HTTPVersion,
}

impl<'a> HTTPEncodable<'a> for RequestLine<'a> {
  fn http_encode(&self, encoding: &mut String) -> anyhow::Result<()> {
    write!(
      encoding,
      "{} {} {}",
      self.method.as_ref(),
      self.request_uri.to_str(),
      self.protocol_version.as_ref()
    )?;

    Ok(())
  }

  fn http_decode(encoding: &'a str) -> anyhow::Result<Self> {
    let mut parts = encoding.split(' ');

    let method: HTTPMethod = parts
      .next()
      .ok_or_else(|| anyhow!("method not found"))?
      .parse()
      .map_err(|_| anyhow!("Unknown method"))?;

    let request_uri = HTTPRequestURI::from(
      parts
        .next()
        .ok_or_else(|| anyhow!("request URI not found"))?,
    );

    let protocol_version = parts
      .next()
      .ok_or_else(|| anyhow!("protocol version not found"))?
      .parse()
      .map_err(|_| anyhow!("Unknown / un-supported protocol version"))?;

    Ok(Self {
      method,
      request_uri,
      protocol_version,
    })
  }
}

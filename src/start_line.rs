use {
  crate::{
    message::HTTPEncodable, method::HTTPMethod, request_uri::HTTPRequestURI,
    status_code::HTTPStatusCode, utils::ToStr, version::HTTPVersion,
  },
  anyhow::anyhow,
  std::fmt::Write,
  typed_builder::TypedBuilder,
};

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
  pub requestURI: HTTPRequestURI<'r>,

  #[builder(default)]
  pub protocolVersion: HTTPVersion,
}

impl<'a> HTTPEncodable<'a> for RequestLine<'a> {
  fn httpEncode(&self, encoding: &mut String) -> anyhow::Result<()> {
    write!(
      encoding,
      "{} {} {}",
      self.method.as_ref(),
      self.requestURI.to_str(),
      self.protocolVersion.as_ref()
    )?;

    Ok(())
  }

  fn httpDecode(encoding: &'a str) -> anyhow::Result<Self> {
    let mut parts = encoding.split(' ');

    let method: HTTPMethod = parts
      .next()
      .ok_or_else(|| anyhow!("method not found"))?
      .parse()
      .map_err(|_| anyhow!("Unknown method"))?;

    let requestURI = HTTPRequestURI::from(
      parts
        .next()
        .ok_or_else(|| anyhow!("request URI not found"))?,
    );

    let protocolVersion = parts
      .next()
      .ok_or_else(|| anyhow!("protocol version not found"))?
      .parse()
      .map_err(|_| anyhow!("Unknown / un-supported protocol version"))?;

    Ok(Self {
      method,
      requestURI,
      protocolVersion,
    })
  }
}

#[derive(TypedBuilder)]
pub struct StatusLine {
  #[builder(default)]
  pub protocolVersion: HTTPVersion,

  // A 3-digit integer result code of the attempt to understand and satisfy the request.
  #[builder(default)]
  pub statusCode: HTTPStatusCode,
  //
  // Each status has a corresponding reason phrase.
  // The client is not required to examine or display the reason phrase.
  // pub reasonPhrase: String,
}

impl<'a> HTTPEncodable<'a> for StatusLine {
  fn httpEncode(&self, encoding: &mut String) -> anyhow::Result<()> {
    write!(
      encoding,
      "{} {} {}",
      self.protocolVersion.as_ref(),
      self.statusCode.clone() as u16,
      self.statusCode.getReasonPhrase()
    )?;

    Ok(())
  }

  fn httpDecode(_encoding: &str) -> anyhow::Result<Self> {
    unimplemented!()
  }
}

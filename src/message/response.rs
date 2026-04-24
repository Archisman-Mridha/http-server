use {
  crate::message::{status_code::HTTPStatusCode, version::HTTPVersion, HTTPEncodable, HTTPMessage},
  std::fmt::Write,
  typed_builder::TypedBuilder,
};

pub type HTTPResponse<'http_response> = HTTPMessage<'http_response, StatusLine>;

impl HTTPResponse<'_> {
  pub fn set_body(&mut self, body: &str) {
    self.body = Some(body.to_string());
    self.headers.insert("Content-Type", "text/plain");
  }

  pub fn set_status_code(&mut self, status_code: HTTPStatusCode) {
    self.start_line.status_code = status_code;
  }
}

#[derive(TypedBuilder)]
pub struct StatusLine {
  #[builder(default)]
  pub protocol_version: HTTPVersion,

  // A 3-digit integer result code of the attempt to understand and satisfy the request.
  #[builder(default)]
  pub status_code: HTTPStatusCode,
  //
  // Each status has a corresponding reason phrase.
  // The client is not required to examine or display the reason phrase.
  // pub reasonPhrase: String,
}

impl<'a> HTTPEncodable<'a> for StatusLine {
  fn http_encode(&self, encoding: &mut String) -> anyhow::Result<()> {
    write!(
      encoding,
      "{} {} {}",
      self.protocol_version.as_ref(),
      self.status_code as u16,
      self.status_code.get_reason_phrase()
    )?;

    Ok(())
  }

  fn http_decode(_encoding: &str) -> anyhow::Result<Self> {
    unimplemented!("unnecessary")
  }
}

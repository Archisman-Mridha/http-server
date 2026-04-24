use crate::message::{HTTPRequest, HTTPResponse};

pub trait HTTPRouter: Sync + Send + 'static {
  fn handle<'connection>(&self, request: &'connection HTTPRequest) -> HTTPResponse<'connection>;
}

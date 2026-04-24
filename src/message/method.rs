use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, PartialEq, Eq, AsRefStr, EnumString)]
pub enum HTTPMethod {
  OPTIONS,
  GET,
  HEAD,
  POST,
  PUT,
  DELETE,
  TRACE,
  CONNECT,
  UNKNOWN,
}

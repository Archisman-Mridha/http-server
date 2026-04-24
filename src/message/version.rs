use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, PartialEq, Eq, AsRefStr, EnumString, Default)]
pub enum HTTPVersion {
  #[default]
  #[strum(serialize = "HTTP/1.1")]
  One,
}

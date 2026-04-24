#[derive(Default, Clone)]
#[repr(u16)]
pub enum HTTPStatusCode {
  // Informational status codes.
  // They represent : Request received, continuing process.
  Continue = 100,
  SwitchingProtocols = 101,

  // Success status codes.
  // They represent : The action was successfully received, understood, and accepted.
  #[default]
  OK = 200,
  Created = 201,
  Accepted = 202,
  NonAuthoritativeInformation = 203,
  NoContent = 204,
  ResetContent = 205,
  PartialContent = 206,

  // Redirection status codes.
  // They represent : Further action must be taken in order to complete the request.
  MultipleChoices = 300,
  MovedPermanently = 301,
  Found = 302,
  SeeOther = 303,
  NotModified = 304,
  UseProxy = 305,
  TemporaryRedirect = 307,

  // Client error status codes.
  // They represent : The request contains bad syntax or cannot be fulfilled.
  BadRequest = 400,
  Unauthorized = 401,
  PaymentRequired = 402,
  Forbidden = 403,
  NotFound = 404,
  MethodNotAllowed = 405,
  NotAcceptable = 406,
  ProxyAuthenticationRequired = 407,
  RequestTimeout = 408,
  Conflict = 409,
  Gone = 410,
  LengthRequired = 411,
  PreconditionFailed = 412,
  RequestEntityTooLarge = 413,
  RequestURITooLong = 414,
  UnsupportedMediaType = 415,
  RequestedRangeNotSatisfiable = 416,
  ExpectationFailed = 417,

  // Server error status codes.
  // They represent : The server failed to fulfill an apparently valid request.
  InternalServerError = 500,
  NotImplemented = 501,
  BadGateway = 502,
  ServiceUnavailable = 503,
  GatewayTimeout = 504,
  HTTPVersionNotSupported = 505,
}

impl HTTPStatusCode {
  pub fn getReasonPhrase(&self) -> &str {
    match self {
      Self::Continue => "Continue",
      Self::SwitchingProtocols => "Switching Protocols",
      Self::OK => "OK",
      Self::Created => "Created",
      Self::Accepted => "Accepted",
      Self::NonAuthoritativeInformation => "Non-Authoritative Information",
      Self::NoContent => "No Content",
      Self::ResetContent => "Reset Content",
      Self::PartialContent => "Partial Content",
      Self::MultipleChoices => "Multiple Choices",
      Self::MovedPermanently => "Moved Permanently",
      Self::Found => "Found",
      Self::SeeOther => "See Other",
      Self::NotModified => "Not Modified",
      Self::UseProxy => "Use Proxy",
      Self::TemporaryRedirect => "Temporary Redirect",
      Self::BadRequest => "Bad Request",
      Self::Unauthorized => "Unauthorized",
      Self::PaymentRequired => "Payment Required",
      Self::Forbidden => "Forbidden",
      Self::NotFound => "Not Found",
      Self::MethodNotAllowed => "Method Not Allowed",
      Self::NotAcceptable => "Not Acceptable",
      Self::ProxyAuthenticationRequired => "Proxy Authentication Required",
      Self::RequestTimeout => "Request Time-out",
      Self::Conflict => "Conflict",
      Self::Gone => "Gone",
      Self::LengthRequired => "Length Required",
      Self::PreconditionFailed => "Precondition Failed",
      Self::RequestEntityTooLarge => "Request Entity Too Large",
      Self::RequestURITooLong => "Request-URI Too Large",
      Self::UnsupportedMediaType => "Unsupported Media Type",
      Self::RequestedRangeNotSatisfiable => "Requested range Not Satisfiable",
      Self::ExpectationFailed => "Expectation Failed",
      Self::InternalServerError => "Internal Server Error",
      Self::NotImplemented => "Not Implemented",
      Self::BadGateway => "Bad Gateway",
      Self::ServiceUnavailable => "Service Unavailable",
      Self::GatewayTimeout => "Gateway Time-out",
      Self::HTTPVersionNotSupported => "HTTP Version not supported",
    }
  }
}

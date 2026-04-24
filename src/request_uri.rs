use crate::utils::ToStr;

#[derive(Debug, PartialEq, Eq)]
pub enum HTTPRequestURI<'a> {
  /*
    Means that the request does not apply to a particular resource, but to the server itself, and
    is only allowed when the method used does not necessarily apply to a resource.

    Example : OPTIONS * HTTP/1.1.
  */
  Asterisk,

  /*
    Required when the request is being made to a proxy.
    The proxy is requested to forward the request (or service it from a valid cache), and return the
    response.

    Example : GET http://www.w3.org/pub/WWW/TheProject.html HTTP/1.1

    To allow for transition to absoluteURIs in all requests in future versions of HTTP, all HTTP/1.1
    servers MUST accept the absoluteURI form in requests, even though HTTP/1.1 clients will only
    generate them in requests to proxies.
  */
  AbsoluteURI(&'a str),

  /*
    Used to identify a resource on an origin server or gateway.
    In this case, the absolute path of the URI must be transmitted as the Request-URI, and the
    network location of the URI (authority) must be transmitted in a Host header field.

    Example : GET /pub/WWW/TheProject.html HTTP/1.1
              Host: www.w3.org

    The absolute path cannot be empty. If none is present in the original URI, it must be given as
    "/" (the server root).

    A transparent proxy must not rewrite the absolute path part of the received Request-URI when
    forwarding it to the next inbound server, except as noted above to replace a null absolute path
    with "/".
  */
  AbsolutePath(&'a str),
}

impl<'a> From<&'a str> for HTTPRequestURI<'a> {
  fn from(requestURI: &'a str) -> Self {
    match requestURI {
      "*" => Self::Asterisk,
      absoluteURI if isAbsoluteURI(requestURI) => Self::AbsoluteURI(absoluteURI),
      absolutePath => Self::AbsolutePath(absolutePath),
    }
  }
}

// Returns whether the given request URI string is an absolute URI.
fn isAbsoluteURI(requestURI: &str) -> bool {
  requestURI.starts_with("http://") || requestURI.starts_with("https://")
}

impl ToStr for HTTPRequestURI<'_> {
  fn to_str(&self) -> &str {
    match self {
      Self::Asterisk => "*",
      Self::AbsoluteURI(absoluteURI) => absoluteURI,
      Self::AbsolutePath(absolutePath) => absolutePath,
    }
  }
}

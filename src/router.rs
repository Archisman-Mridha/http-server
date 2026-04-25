use {
  crate::message::{
    request::HTTPRequest,
    request_uri::HTTPRequestURI,
    response::{HTTPResponse, StatusLine},
    status_code::HTTPStatusCode,
    HTTPMessage,
  },
  std::collections::HashMap,
};

pub trait HTTPRouter: Sync + Send + 'static {
  fn handle<'connection>(&self, request: &'connection HTTPRequest) -> HTTPResponse<'connection>;
}

pub type RequestHandler =
  Box<dyn for<'connection> Fn(&'connection HTTPRequest) -> HTTPResponse<'connection> + Send + Sync>;

#[derive(Default)]
pub struct Trie<'trie> {
  root: TrieNode<'trie>,
}

#[derive(Default)]
pub struct TrieNode<'trie_node> {
  handler: Option<RequestHandler>,

  children: HashMap<&'trie_node str, TrieNode<'trie_node>>,
}

impl<'trie> Trie<'trie> {
  pub fn register(&mut self, route: &'trie str, handler: RequestHandler) {
    let route = route.strip_suffix('/').unwrap_or(route);
    let route_segments: Vec<&str> = route.split('/').collect();

    let mut current_trie_node = &mut self.root;
    for route_segment in route_segments.iter() {
      current_trie_node = current_trie_node.children.entry(route_segment).or_default();
    }
    current_trie_node.handler = Some(handler);
  }
}

impl<'trie: 'static> HTTPRouter for Trie<'trie> {
  fn handle<'connection>(&self, request: &'connection HTTPRequest) -> HTTPResponse<'connection> {
    let route = match request.start_line.request_uri {
      HTTPRequestURI::AbsolutePath(route) => route,

      _ => {
        return HTTPMessage::builder()
          .start_line(
            StatusLine::builder()
              .status_code(HTTPStatusCode::NotFound)
              .build(),
          )
          .build()
      }
    };

    let route = route.strip_suffix('/').unwrap_or(route);
    let route_segments: Vec<&str> = route.split('/').collect();

    let mut current_node = &self.root;
    let mut handler: Option<&RequestHandler> = None;

    for segment in route_segments.iter() {
      if current_node.handler.is_some() {
        handler = current_node.handler.as_ref();
      }

      match current_node.children.get(*segment) {
        Some(next) => current_node = next,
        None => break,
      }
    }
    if current_node.handler.is_some() {
      handler = current_node.handler.as_ref();
    }

    match handler {
      Some(handler) => handler(request),

      _ => HTTPMessage::builder()
        .start_line(
          StatusLine::builder()
            .status_code(HTTPStatusCode::NotFound)
            .build(),
        )
        .build(),
    }
  }
}

use {
  archttp::{
    message::{
      method::HTTPMethod,
      request::HTTPRequest,
      response::{HTTPResponse, StatusLine},
      status_code::HTTPStatusCode,
      HTTPMessage,
    },
    server::{HTTPServer, HTTPServerCore},
    utils::ToStr,
  },
  clap::Parser,
  std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
  },
  typed_builder::TypedBuilder,
};

#[derive(Parser)]
struct Args {
  #[arg(long, default_value = "/tmp")]
  directory: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let http_router = HTTPRouter::builder().directory(args.directory).build();

  let http_server_core = HTTPServerCore::builder().router(http_router).build();

  HTTPServer::new(http_server_core).run().await
}

const ECHO_ROUTE_PREFIX: &str = "/echo/";
const USER_AGENT_ROUTE: &str = "/user-agent";
const FILES_ROUTE_PREFIX: &str = "/files/";

#[derive(TypedBuilder)]
struct HTTPRouter {
  directory: String,
}

impl archttp::router::HTTPRouter for HTTPRouter {
  fn handle<'connection>(&self, request: &'connection HTTPRequest) -> HTTPResponse<'connection> {
    let path = request.start_line.request_uri.to_str();
    match path {
      _ if path.starts_with(ECHO_ROUTE_PREFIX) => echo_route_handler(request),

      USER_AGENT_ROUTE => user_agent_route_handler(request),

      _ if path.starts_with(FILES_ROUTE_PREFIX) => files_route_handler(request, &self.directory),

      "/" => HTTPMessage::builder()
        .start_line(StatusLine::builder().build())
        .build(),

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

fn echo_route_handler<'a>(request: &'a HTTPRequest) -> HTTPResponse<'a> {
  let path = request.start_line.request_uri.to_str();

  let echo_message = path.strip_prefix(ECHO_ROUTE_PREFIX).unwrap();

  let mut http_response = HTTPMessage::builder()
    .start_line(StatusLine::builder().build())
    .build();
  http_response.set_body(echo_message);
  http_response
}

fn user_agent_route_handler<'a>(request: &'a HTTPRequest) -> HTTPResponse<'a> {
  let user_agent = request.headers.get("User-Agent").unwrap();

  let mut http_response = HTTPMessage::builder()
    .start_line(StatusLine::builder().build())
    .build();
  http_response.set_body(user_agent.as_ref());
  http_response
}

fn files_route_handler<'a>(request: &'a HTTPRequest, directory: &'_ str) -> HTTPResponse<'a> {
  let path = request.start_line.request_uri.to_str();

  let file_name = path.strip_prefix(FILES_ROUTE_PREFIX).unwrap();
  let file_path = Path::new(directory).join(file_name);

  let mut http_response = HTTPMessage::builder()
    .start_line(StatusLine::builder().build())
    .build();

  match request.start_line.method {
    HTTPMethod::GET => match fs::read_to_string(file_path) {
      Ok(file_content) => {
        http_response.set_body(&file_content);
        http_response
          .headers
          .insert("Content-Type", "application/octet-stream");
      }

      Err(_) => http_response.start_line.status_code = HTTPStatusCode::NotFound,
    },

    HTTPMethod::POST => {
      let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)
        .unwrap();
      write!(file, "{}", request.body.as_deref().unwrap_or("")).unwrap();

      http_response.start_line.status_code = HTTPStatusCode::Created;
    }

    _ => http_response.start_line.status_code = HTTPStatusCode::MethodNotAllowed,
  }

  http_response
}

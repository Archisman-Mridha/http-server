#![allow(non_snake_case)]

use {
  clap::Parser,
  std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
  },
  typed_builder::TypedBuilder,
  vertex::{
    message::{HTTPMessage, HTTPRequest, HTTPResponse},
    method::HTTPMethod,
    server::{HTTPServer, HTTPServerCore},
    start_line::StatusLine,
    status_code::HTTPStatusCode,
    utils::ToStr,
  },
};

#[derive(Parser)]
struct Args {
  #[arg(long, default_value = "/tmp")]
  directory: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let httpRouter = HTTPRouter::builder().directory(args.directory).build();

  let httpServerCore = HTTPServerCore::builder().router(httpRouter).build();

  HTTPServer::new(httpServerCore).run().await
}

const ECHO_ROUTE_PREFIX: &str = "/echo/";
const USER_AGENT_ROUTE: &str = "/user-agent";
const FILES_ROUTE_PREFIX: &str = "/files/";

#[derive(TypedBuilder)]
struct HTTPRouter {
  directory: String,
}

impl vertex::router::HTTPRouter for HTTPRouter {
  fn handle<'connection>(&self, request: &'connection HTTPRequest) -> HTTPResponse<'connection> {
    let path = request.startLine.requestURI.to_str();
    match path {
      _ if path.starts_with(ECHO_ROUTE_PREFIX) => echoRouteHandler(request),

      USER_AGENT_ROUTE => userAgentRouteHandler(request),

      _ if path.starts_with(FILES_ROUTE_PREFIX) => filesRouteHandler(request, &self.directory),

      "/" => HTTPMessage::builder()
        .startLine(StatusLine::builder().build())
        .build(),

      _ => HTTPMessage::builder()
        .startLine(
          StatusLine::builder()
            .statusCode(HTTPStatusCode::NotFound)
            .build(),
        )
        .build(),
    }
  }
}

fn echoRouteHandler<'a>(request: &'a HTTPRequest) -> HTTPResponse<'a> {
  let path = request.startLine.requestURI.to_str();

  let echoMessage = path.strip_prefix(ECHO_ROUTE_PREFIX).unwrap();

  let mut httpResponse = HTTPMessage::builder()
    .startLine(StatusLine::builder().build())
    .build();
  httpResponse.setBody(echoMessage);
  httpResponse
}

fn userAgentRouteHandler<'a>(request: &'a HTTPRequest) -> HTTPResponse<'a> {
  let userAgent = request.headers.get("User-Agent").unwrap();

  let mut httpResponse = HTTPMessage::builder()
    .startLine(StatusLine::builder().build())
    .build();
  httpResponse.setBody(userAgent);
  httpResponse
}

fn filesRouteHandler<'a>(request: &'a HTTPRequest, directory: &'_ str) -> HTTPResponse<'a> {
  let path = request.startLine.requestURI.to_str();

  let fileName = path.strip_prefix(FILES_ROUTE_PREFIX).unwrap();
  let filePath = Path::new(directory).join(fileName);

  let mut httpResponse = HTTPMessage::builder()
    .startLine(StatusLine::builder().build())
    .build();

  match request.startLine.method {
    HTTPMethod::GET => match fs::read_to_string(filePath) {
      Ok(fileContent) => {
        httpResponse.setBody(fileContent.leak());
        httpResponse
          .headers
          .insert("Content-Type", "application/octet-stream");
      }

      Err(_) => httpResponse.startLine.statusCode = HTTPStatusCode::NotFound,
    },

    HTTPMethod::POST => {
      let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filePath)
        .unwrap();
      write!(file, "{}", request.body.unwrap_or("")).unwrap();

      httpResponse.startLine.statusCode = HTTPStatusCode::Created;
    }

    _ => httpResponse.startLine.statusCode = HTTPStatusCode::MethodNotAllowed,
  }

  httpResponse
}

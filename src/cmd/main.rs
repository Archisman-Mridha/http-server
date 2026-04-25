use {
  archttp::{
    message::{
      method::HTTPMethod,
      request::HTTPRequest,
      response::{HTTPResponse, StatusLine},
      status_code::HTTPStatusCode,
      HTTPMessage,
    },
    router::Trie,
    server::{HTTPServer, HTTPServerCore},
    utils::ToStr,
  },
  clap::Parser,
  std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::Arc,
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

  let mut http_router = Trie::default();

  http_router.register("/ping", Box::new(ping_route_handler));
  http_router.register("/echo", Box::new(echo_route_handler));
  http_router.register("/user-agent", Box::new(user_agent_route_handler));

  let directory = Arc::new(args.directory);
  http_router.register(
    "/files",
    Box::new({
      let directory = Arc::clone(&directory);
      move |request: &HTTPRequest| -> HTTPResponse { files_route_handler(request, &directory) }
    }),
  );

  let http_server_core = HTTPServerCore::builder().router(http_router).build();

  HTTPServer::new(http_server_core).run().await
}

const ECHO_ROUTE_PREFIX: &str = "/echo/";
const FILES_ROUTE_PREFIX: &str = "/files/";

fn ping_route_handler<'a>(_: &'a HTTPRequest) -> HTTPResponse<'a> {
  let mut http_response = HTTPMessage::builder()
    .start_line(StatusLine::builder().build())
    .build();
  http_response.set_body("PONG");
  http_response
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

fn files_route_handler<'a>(request: &'a HTTPRequest, directory: &str) -> HTTPResponse<'a> {
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

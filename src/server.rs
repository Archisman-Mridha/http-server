use {
  crate::{
    message::{request::HTTPRequest, response::HTTPResponse, HTTPEncodable, HTTPMessage},
    router::HTTPRouter,
  },
  flate2::{write::GzEncoder, Compression},
  std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
  },
  tokio::task::JoinHandle,
  typed_builder::TypedBuilder,
};

pub struct HTTPServer<R>
where
  R: HTTPRouter,
{
  core: Arc<HTTPServerCore<R>>,
}

impl<R> HTTPServer<R>
where
  R: HTTPRouter,
{
  pub fn new(core: HTTPServerCore<R>) -> Self {
    Self {
      core: Arc::new(core),
    }
  }

  pub async fn run(&self) -> anyhow::Result<()> {
    println!("Starting HTTP server at {}", self.core.address);

    /*
      The TCP-IP protocol :

      The Internet Protocol (IP) is the address system of the Internet and has the core function of
      delivering packets of information from a source device to a target device. IP is the primary
      way in which network connections are made, and it establishes the basis of the Internet.

      IP does not handle packet ordering or error checking. Such functionality requires another
      protocol, often the Transmission Control Protocol (TCP).

      IP makes sure the packets arrive at their destination address. TCP can be thought of as the
      assembler on the other side who puts the packets together in the right order, asks for missing
      packets to be resent, and lets the sender know the packet has been received.

      REFERENCE : https://www.cloudflare.com/en-ca/learning/ddos/glossary/tcp-ip/.
    */
    let listener = TcpListener::bind(self.core.address.clone()).unwrap();

    let mut thread_handles: Vec<JoinHandle<()>> = Vec::new();

    for stream in listener.incoming() {
      match stream {
        Ok(connection) => {
          let http_server_core = self.core.clone();

          let thread_handle = tokio::spawn(async move {
            if let Err(error) = http_server_core.handle_connection(connection).await {
              eprintln!("Failed handling connection : {error}")
            }
          });
          thread_handles.push(thread_handle);
        }
        Err(error) => println!("Connection error : {error}"),
      }
    }

    // Wait for all the requests to be processed.
    for thread_handle in thread_handles {
      let _ = thread_handle.await;
    }
    Ok(())
  }
}

#[derive(TypedBuilder)]
pub struct HTTPServerCore<R>
where
  R: HTTPRouter,
{
  #[builder(default_code = "String::from(\"127.0.0.1:4221\")")]
  pub address: String,

  pub router: R,
}

impl<R> HTTPServerCore<R>
where
  R: HTTPRouter,
{
  async fn handle_connection(&self, mut connection: TcpStream) -> anyhow::Result<()> {
    let mut buf_reader = BufReader::new(&mut connection);

    let mut encoded_http_request = String::new();

    // Read until the end of request header section.
    loop {
      buf_reader.read_line(&mut encoded_http_request)?;
      if encoded_http_request.ends_with("\r\n\r\n") {
        break;
      }
    }

    let mut http_request: HTTPRequest = HTTPMessage::http_decode(&encoded_http_request)?;

    // Read request body (if present).
    if let Some(content_length) = http_request.headers.get("Content-Length") {
      let content_length = content_length.parse().unwrap();

      let mut http_request_body = vec![0; content_length];
      buf_reader.read_exact(&mut http_request_body).unwrap();

      let http_request_body = String::from_utf8(http_request_body).unwrap();
      http_request.body = Some(http_request_body);
    }

    println!("Received encoded HTTP request : \n{encoded_http_request}");

    let encoded_http_response = self.handle_request(http_request)?;

    println!("Sending encoded HTTP response : \n{encoded_http_response}");
    connection.write_all(encoded_http_response.as_bytes())?;
    Ok(())
  }

  fn handle_request(&self, http_request: HTTPRequest) -> anyhow::Result<String> {
    let mut http_response: HTTPResponse = self.router.handle(&http_request);

    if http_response.body.is_some() {
      if let Some(client_supported_encodings) = http_request.headers.get("Accept-Encoding") {
        let client_supported_encodings = client_supported_encodings.split(", ");
        for client_supported_encoding in client_supported_encodings {
          #[allow(clippy::single_match)]
          match client_supported_encoding {
            "gzip" => {
              http_response.headers.insert("Content-Encoding", "gzip");

              let mut gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
              gzip_encoder
                .write_all(http_response.body.as_deref().unwrap().as_bytes())
                .unwrap();
              let gzip_encoding = gzip_encoder.finish().unwrap();

              http_response.set_body(&hex::encode(&gzip_encoding));

              break;
            }

            _ => {}
          }
        }
      }
    }

    let mut encoded_http_response = String::new();
    http_response.http_encode(&mut encoded_http_response)?;
    Ok(encoded_http_response)
  }
}

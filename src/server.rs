use {
  crate::{
    message::{HTTPEncodable, HTTPMessage, HTTPRequest, HTTPResponse},
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

    let mut threadHandles: Vec<JoinHandle<()>> = Vec::new();

    for stream in listener.incoming() {
      match stream {
        Ok(connection) => {
          let httpServerCore = self.core.clone();

          let threadHandle = tokio::spawn(async move {
            if let Err(error) = httpServerCore.handleConnection(connection).await {
              eprintln!("Failed handling connection : {error}")
            }
          });
          threadHandles.push(threadHandle);
        }
        Err(error) => println!("Connection error : {error}"),
      }
    }

    // Wait for all the requests to be processed.
    for threadHandle in threadHandles {
      let _ = threadHandle.await;
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
  async fn handleConnection(&self, mut connection: TcpStream) -> anyhow::Result<()> {
    let mut bufReader = BufReader::new(&mut connection);

    let mut encodedHTTPRequest = String::new();

    // Read until the end of request header section.
    loop {
      bufReader.read_line(&mut encodedHTTPRequest)?;
      if encodedHTTPRequest.ends_with("\r\n\r\n") {
        break;
      }
    }

    let mut httpRequest: HTTPRequest = HTTPMessage::httpDecode(&encodedHTTPRequest)?;

    // Read request body (if present).
    if let Some(contentLength) = httpRequest.headers.get("Content-Length") {
      let contentLength = contentLength.parse().unwrap();

      let mut httpRequestBody = vec![0; contentLength];
      bufReader.read_exact(&mut httpRequestBody).unwrap();

      let httpRequestBody = String::from_utf8_lossy(&httpRequestBody).to_string();
      httpRequest.body = Some(httpRequestBody.leak());
    }

    println!("Received encoded HTTP request : \n{encodedHTTPRequest}");

    let encodedHTTPResponse = &self.handleRequest(httpRequest)?;

    println!("Sending encoded HTTP response : \n{encodedHTTPResponse}");
    connection.write_all(encodedHTTPResponse.as_bytes())?;
    Ok(())
  }

  fn handleRequest<'connection>(
    &self,
    httpRequest: HTTPRequest,
  ) -> anyhow::Result<&'connection str> {
    let mut httpResponse: HTTPResponse = self.router.handle(&httpRequest);

    if httpResponse.body.is_some() {
      if let Some(clientSupportedEncodings) = httpRequest.headers.get("Accept-Encoding") {
        let clientSupportedEncodings = clientSupportedEncodings.split(", ");
        for clientSupportedEncoding in clientSupportedEncodings {
          #[allow(clippy::single_match)]
          match clientSupportedEncoding {
            "gzip" => {
              httpResponse.headers.insert("Content-Encoding", "gzip");

              let mut gzipEncoder = GzEncoder::new(Vec::new(), Compression::default());
              gzipEncoder
                .write_all(httpResponse.body.unwrap().as_bytes())
                .unwrap();
              let gzipEncoding = gzipEncoder.finish().unwrap();

              httpResponse.setBody(hex::encode(&gzipEncoding).leak());

              break;
            }

            _ => {}
          }
        }
      }
    }

    let mut encodedHTTPResponse = String::new();
    httpResponse.httpEncode(&mut encodedHTTPResponse)?;
    Ok(encodedHTTPResponse.leak())
  }
}

//!  A simple HTTP client, using mio for non-blocking I/O.

use std::io::{ErrorKind, Read, Write};

use crate::future::{Future, PollState};

/// A simple HTTP client.
pub struct Http;

impl Http {
    /// A get request via the HTTP client will
    /// return a HTTPGetFuture. This represents
    /// some furniture operation that will resolve
    /// and yield and output from the future.
    ///
    /// In this case, the output type has specifies that
    /// the Future should yield a String.
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

/// A future that represents a HTTP GET request.
///
/// The result from polling this future to completion
/// is the response to the HTTP get request.
///
/// It also holds some internal state for the Future, that
/// is available between calls to poll.
struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    fn new(path: &str) -> Self {
        Self {
            stream: None,
            buffer: Vec::new(),
            path: path.to_string(),
        }
    }

    fn write_request(&mut self) {
        let std_stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        std_stream.set_nonblocking(true).unwrap();

        // mio wraps the std stream
        let mut stream = mio::net::TcpStream::from_std(std_stream);

        stream.write_all(&get_req(&self.path)).unwrap();

        self.stream = Some(stream);
    }
}

/// This is a leaf-future, hence it's implementation does
/// not necessarily correspond to those of a state machine,
/// like most non-leaf coroutines would.
///
/// it does not yet use an event-queue though, but uses a mio TcpStream,
/// which supports Event, Token, Registry, Source, etc.
impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("FIRST POLL - STARTING OPERATION");
            self.write_request();
            return PollState::NotReady;
        }

        let mut buff = vec![0_u8; 4096]; // 4KB buffer

        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer);
                    return PollState::Ready(s.to_string());
                }
                Ok(n) => {
                    self.buffer.extend(&buff[..n]);
                    continue;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // println!("WOULD BLOCK");
                    return PollState::NotReady;
                }

                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("IO error: {:?}", e),
            }
        }
    }
}

fn get_req(path: &str) -> Vec<u8> {
    let req = format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    );

    req.into_bytes()
}

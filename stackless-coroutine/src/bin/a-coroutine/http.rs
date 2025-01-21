//! Makes only GET requests to the delayserver in `rust-async-utils`
#![allow(unused)]
// traits and types from reading from a IO source
use std::{
    io::{ErrorKind, Read, Write},
    os::unix::raw::off_t,
};

use crate::future::{Future, PollState};

static DELAYSERVER: &str = "127.0.0.1:8080";

/// The main http client responsible for I/O operations via kernel
///
/// While not required, we can add state to it at a later date + good for encapsulating
/// functionality related to making http requests to a server.
pub struct Http;

pub struct Response {
    pub body: String,
}

impl Http {
    /// Returns a future that yields the response of the HTTP request
    pub fn get(path: &str) -> impl Future<Output = Response> {
        HttpGetFuture::new(path)
    }
}

/// A Leaf Future
struct HttpGetFuture {
    /// Optional since we do not connect on instantiation of HttpGetFuture
    stream: Option<mio::net::TcpStream>,
    /// data read from TCP stream is placed here
    buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    fn new(path: &str) -> Self {
        Self {
            // do not connect yet, only on first poll
            stream: None,
            buffer: Vec::new(),
            path: path.to_string(),
        }
    }

    /// Makes a non-blocking write request to the delayserver
    /// and stores the created stream on the future.
    fn write_request(&mut self) {
        // Create a standard library stream first and wrap it in mio stream
        let stream = std::net::TcpStream::connect(DELAYSERVER).unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);

        let req = get_req(&self.path);

        // non-blocking IO operation
        stream.write_all(&req).unwrap();

        // store stream on future
        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = Response;
    /// Below can be viewed as a simple state machine with 3 possible states.
    ///
    /// 1. Not Started: indicated by self.stream being None.
    /// 2. Pending: indicatd by self.stream being Some and a read to `stream.read`
    ///    returning `ErrorKind::WouldBlock`.
    /// 3. Resolved, indicated by self.stream being Some and `stream.read`
    ///    returning 0 bytes.
    fn poll(&mut self) -> PollState<Self::Output> {
        // If stream is none, this is first time we are polling the future, so
        // "progressing" the future, means making a request to the delayserver.
        if self.stream.is_none() {
            // Send GET request and store created stream on future.
            println!("FIRST POLL - STARTING OPERATION - Make GET REQUEST");
            self.write_request();
            return PollState::NotReady;
        }

        // Reach here if this is not first poll on the future.
        // "Progressing" the future means waiting / checking if response is ready.
        let mut buff = vec![0u8; 4096]; // 4Kb buffer

        // we keep trying to read from stream until we reach end
        // or if operation would block
        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    // we have reached end of buffer
                    let response = Response {
                        body: String::from_utf8_lossy(&*self.buffer).to_string(),
                    };

                    return PollState::Ready(response);
                }
                Ok(n) => {
                    // we have read N bytes, extend buffer on future with temporary buffer.

                    self.buffer.extend_from_slice(&buff[..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    // we would block, return NotReady
                    // also reach here if we are interrupted
                    // return PollState::NotReady;
                    break PollState::NotReady; // break and retun value from `loop`
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    // try reading again
                    continue;
                }
                // We do no error handling, so all we do is panic in below situation.
                Err(e) => panic!("IO Error: {e:?}"),
            }
        }
    }
}

/// Helper function to write actual GET request as a stream of bytes
fn get_req(path: &str) -> Vec<u8> {
    let req = format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    );

    req.into_bytes()
}

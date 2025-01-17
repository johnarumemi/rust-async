//!  A simple HTTP client, using mio for non-blocking I/O.

use std::io::{ErrorKind, Read, Write};

use mio::Interest;

use crate::future::{Future, PollState};
use crate::runtime::{self, reactor, Waker};

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
    // This gives the HttpGetFuture an identity. Which we need, since it's
    // the Source of events we want to track with our Reactor.
    id: usize,
}

impl HttpGetFuture {
    fn new(path: &str) -> Self {
        Self {
            stream: None,
            buffer: Vec::new(),
            path: path.to_string(),
            // Get the id for this future from the Reactor.
            id: reactor().next_id(),
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

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("FIRST POLL - STARTING OPERATION");
            self.write_request();

            // register interest in READABLE events for streams file descriptor
            // with our runtimes registry / event queue

            // Note that our stream is a mio TcpStream, hence implements Source trait.
            // we can pass in a &mut Source and it will know how to extract as_raw_fd
            //
            // ```rust
            // use std::os::fd::AsRawFd;
            // self.stream.map(|v| v.as_raw_fd()).unwrap();
            // ```

            let stream = self.stream.as_mut().unwrap();

            // register interest
            reactor().register(stream, Interest::READABLE, self.id);

            // Set waker for this future
            reactor().set_waker(waker, self.id)

            // NOTE that we poll TcpStream immediately on the first poll to this future.
        }

        let mut buff = vec![0_u8; 4096]; // 4KB buffer

        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer);
                    println!("Completed Read");
                    // deregister interest
                    reactor().deregister(self.stream.as_mut().unwrap(), self.id);
                    return PollState::Ready(s.to_string());
                }
                Ok(n) => {
                    println!("Reading");
                    self.buffer.extend(&buff[..n]);
                    continue;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    println!("WOULD BLOCK");
                    // update waker
                    reactor().set_waker(waker, self.id);
                    return PollState::NotReady;
                }

                Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                    println!("INTERRUPTED");
                    continue;
                }
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

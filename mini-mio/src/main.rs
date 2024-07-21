#![allow(dead_code, unused)]

use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use std::{
    io::{self, Read, Result, Write},
    net::TcpStream,
};

mod ffi;
mod poll;

use ffi::Event;
use poll::Poll;

fn main() -> Result<()> {
    // Create a new event queue
    let mut poll = Poll::new()?;
    let num_events = 5; // max events we are interested in

    let mut streams = vec![];
    let socket_addr = "localhost:8080";
    let mut handled_ids: HashSet<usize> = HashSet::new();

    for i in 0..num_events {
        println!("-- Starting Request {i} --\n");
        // first request has longest timeout, so expect
        // responses to arrive in reverse order.
        let delay = (num_events - i) * 1000;
        println!("Delay: {} ms, for event i = {}", delay, i);
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let mut stream = TcpStream::connect(socket_addr)?;

        // set non-blocking mode
        stream.set_nonblocking(true)?;

        // send packet across stream / socket (non-blocking mode is enabled atm)
        stream.write_all(&request)?;

        // sleep for a while to simulate network latency
        // and also ensure requests arrive in order in the server
        thread::sleep(Duration::from_millis(50));

        // register interest in being notified when steam is ready to read

        println!("Registering stream {i} with epoll");
        poll.registry().register(
            &stream,                     // source
            i,                           // token
            ffi::EPOLLIN | ffi::EPOLLET, // bitmask for read + edge-triggered
        )?;
        // NOTE following:
        // EPOLLIN  = 00000000000000000000000000000001
        // EPOLLET  = 10000000000000000000000000000000
        // inerests = 10000000000000000000000000000001
        // decimal  = 2147483649
        //
        // hence Event.events = 214748364

        // store stream
        println!("Storing stream...");
        streams.push(stream);

        println!("\n-- Completing Request {i} --\n\n");
    }

    println!("Completed sending all requests and registering streams with epoll\n\n");

    let padding = (0..15).map(|_| "-").collect::<String>();
    let msg = format!("{padding} Starting Event Loop {padding}");
    let boundary = (0..msg.len()).map(|_| "-").collect::<String>();

    println!("\n{boundary}\n{msg}\n{boundary}\n");

    // Now handle read notifications
    let mut handled_events = 0;

    // do below while we haven't got a response from all the requests
    while handled_events < num_events {
        let mut events = Vec::with_capacity(10);

        // register interest in being notified when steam is ready to read
        poll.poll(&mut events, None)?; // block indefinitely

        // reach here when thread is woken up
        if events.is_empty() {
            println!("TIMEOUT OR SPURIOUS WAKEUP EVENT NOTIFICATION");
            continue;
        }

        handled_events += handle_events(&events, &mut streams, &mut handled_ids)?;
    }

    println!("FINISHED PROGRAM");
    Ok(())
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

fn handle_events(
    events: &[Event],
    streams: &mut [TcpStream],
    handled_ids: &mut HashSet<usize>,
) -> Result<usize> {
    let mut handled_events = 0;

    for event in events {
        println!("\n------------------------------------\n");
        ffi::print_event_debug(event);
        ffi::check(event.events as i32);

        let index = event.token();
        let mut data = vec![0u8; 4096]; // 4KB buffer
                                        // let mut data = vec![0u8; 8]; // 4KB buffer

        let mut i = 0_usize;
        let mut txt = String::new();
        let mut new_response = true;

        loop {
            // use a loop to ensure we drain the buffer.
            // This is important for edge-triggered mode, as if the buffer isn't
            // drained, then it will never reset to notify us of new events.
            match streams[index].read(&mut data) {
                Ok(0) => {
                    // read 0 bytes - buffer has been drained successfully

                    // `insert` returns false if the value already existed in the set.
                    if !handled_ids.insert(index) {
                        break;
                    }

                    handled_events += 1;

                    println!(
                        "\n\nBuffer drained after {i} iteration(s), breaking out of loop...\n"
                    );
                    println!("------------------------------------\n");
                    i = 0;
                    new_response = true;
                    break;
                }
                Ok(n) => {
                    // read n bytes
                    let txt = String::from_utf8_lossy(&data[..n]);
                    if new_response {
                        println!("\n--- Response ---");
                        new_response = false;
                    }
                    print!("{txt}");
                    i = i.saturating_add(1);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                // if the read operation is interrupted (e.g. signal from OS), we can continue
                Err(e) if e.kind() == io::ErrorKind::Interrupted => break,
                Err(e) => return Err(e),
            }
        }
    }

    Ok(handled_events)
}

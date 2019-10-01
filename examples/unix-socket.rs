// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows how you can write metrics to a Unix datagram socket instead
// of a UDP socket. This might be useful if you have some sort of Statsd server or
// agent running on the same machine as your application that is exposed via a Unix
// socket.
//
// Note that this example should *not* be used as a model for building a datagram
// server (it has lots of shortcuts and bad practices for the sake of a simple
// example), only as an example for how to use the Cadence UnixMetricSink.

use std::fs;
use std::io::ErrorKind;
use std::os::unix::net::UnixDatagram;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use cadence::prelude::*;
use cadence::{StatsdClient, UnixMetricSink};

const SOCKET_PATH: &str = "/tmp/cadence.sock";

fn main() {
    // "ready" flag to be set by the server after it has created and started
    // listening on a Unix socket. The client waits for this flag to be set
    // before trying to send any metrics (to ensure they aren't lost).
    let ready = Arc::new(AtomicBool::new(false));
    let ready_clone = Arc::clone(&ready);

    // "shutdown" flag to be set by the client to cause the server to stop
    // listening for metrics in a loop and exit its thread at the end of this
    // example.
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    // Make sure to remove the socket if it already exists from a previous run.
    let _ = fs::remove_file(SOCKET_PATH);

    let t = thread::spawn(move || {
        let server_socket = UnixDatagram::bind(SOCKET_PATH).unwrap();
        server_socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        // Fixed size buffer that's enough to hold the metrics we'll send below.
        let mut buf = [0u8; 1024];

        // Indicate that callers can start sending metrics to our socket because
        // it has been created and we've started listening on it (so no metrics
        // will be lost).
        ready_clone.store(true, Ordering::Release);

        loop {
            match server_socket.recv(&mut buf) {
                Ok(v) => {
                    let metric = String::from_utf8_lossy(&buf);
                    println!("Success: got {} bytes from socket: {}", v, metric);
                }
                Err(e) => {
                    // WouldBlock means we hit our receive timeout which is expected.
                    // If the "shutdown" flag has been set by the client they've sent
                    // all the metrics they are going to send and we can shutdown the
                    // server. Otherwise, just ignore the WouldBlock error.
                    if e.kind() == ErrorKind::WouldBlock {
                        if shutdown_clone.load(Ordering::Acquire) {
                            break;
                        }
                    } else {
                        // Some other kind of error besides hitting our receive timeout
                        println!("Error: {} - {:?}", e, e.kind());
                    }
                }
            }
        }
    });

    // Create a client with a UnixMetricSink that writes metrics over a Unix socket
    // to the expected path that the server will be listening on.
    let socket = UnixDatagram::unbound().unwrap();
    let sink = UnixMetricSink::from(SOCKET_PATH, socket);
    let metrics = StatsdClient::from_sink("example.prefix", sink);

    // Busy wait until the server starts listening on the socket and reading metrics.
    while !ready.load(Ordering::Acquire) {
        thread::yield_now();
    }

    metrics.count("example.counter", 1).unwrap();
    metrics.gauge("example.gauge", 5).unwrap();
    metrics.time("example.timer", 32).unwrap();
    metrics.histogram("example.histogram", 22).unwrap();
    metrics.meter("example.meter", 8).unwrap();

    // Indicate to the server that we've sent everything we're going to send and
    // it's OK to shutdown now.
    shutdown.store(true, Ordering::Release);

    // Wait for shutdown to complete and clean up the socket created.
    t.join().unwrap();
    let _ = fs::remove_file(SOCKET_PATH);
}

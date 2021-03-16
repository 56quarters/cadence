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
// In this example we make use of some of the Cadence testing code to cut down
// on the amount of boilerplate we need to write. The server harness here spins
// up a server, runs the provided closure with the path to the socket as an
// argument, then waits for the server to shut down.

use std::os::unix::net::UnixDatagram;

use cadence::prelude::*;
use cadence::test::UnixServerHarness;
use cadence::{StatsdClient, UnixMetricSink};

fn main() {
    let harness = UnixServerHarness::new("unix-socket-example");
    harness.run(
        |s: String| println!("Got {} bytes from socket: {}", s.len(), s),
        |path| {
            let socket = UnixDatagram::unbound().unwrap();
            let sink = UnixMetricSink::from(path, socket);
            let metrics = StatsdClient::from_sink("example.prefix", sink);

            metrics.count("example.counter", 1).unwrap();
            metrics.gauge("example.gauge", 5).unwrap();
            metrics.time("example.timer", 32).unwrap();
            metrics.histogram("example.histogram", 22).unwrap();
            metrics.distribution("example.distribution", 33).unwrap();
            metrics.meter("example.meter", 8).unwrap();
        },
    );
}

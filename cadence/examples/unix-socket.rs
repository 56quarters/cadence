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

use cadence::prelude::*;
use cadence::test::UnixServerHarness;
use cadence::{StatsdClient, UnixMetricSink};
use std::os::unix::net::UnixDatagram;
use std::time::Duration;

fn main() {
    let harness = UnixServerHarness::new("unix-socket-example");
    harness.run(
        |s: String| println!("Got {} bytes from socket: {}", s.len(), s),
        |path| {
            let socket = UnixDatagram::unbound().unwrap();
            let sink = UnixMetricSink::from(path, socket);
            let client = StatsdClient::from_sink("example.prefix", sink);

            client.count("example.counter", 1).unwrap();
            client.gauge("example.gauge", 5).unwrap();
            client.gauge("example.gauge", 5.0).unwrap();
            client.time("example.timer", 32).unwrap();
            client.time("example.timer", Duration::from_millis(32)).unwrap();
            client.histogram("example.histogram", 22).unwrap();
            client.histogram("example.histogram", Duration::from_nanos(22)).unwrap();
            client.histogram("example.histogram", 22.0).unwrap();
            client.distribution("example.distribution", 33).unwrap();
            client.distribution("example.distribution", 33.0).unwrap();
            client.meter("example.meter", 8).unwrap();
            client.set("example.set", 44).unwrap();
        },
    );
}

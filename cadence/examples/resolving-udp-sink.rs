// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows using a UDP sink that automatically re-resolves the address
// of the server periodically.

use cadence::prelude::*;
use cadence::{BufferedUdpMetricSink, StatsdClient, DEFAULT_PORT};
use std::net::UdpSocket;
use std::time::Duration;

fn main() {
    let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::builder()
        .with_capacity(128)
        .with_resolver_period(Duration::from_millis(100))
        .with_resolver_error_handler(|e| {
            eprintln!("failed to resolve server: {}", e);
        })
        .build(("localhost", DEFAULT_PORT), sock)
        .unwrap();

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
}

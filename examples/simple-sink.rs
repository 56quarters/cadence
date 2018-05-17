// Cadence - An extensible Statsd client for Rust!
//
// This file is dual-licensed to the public domain and under the following
// license: you are granted a perpetual, irrevocable license to copy, modify,
// publish, and distribute this file as you see fit.

// This example shows using a very simple UDP sink. This sink will not
// give you any isolation (network calls are done in the calling thread)
// and does not offer the performance of the buffering sink.

extern crate cadence;

use cadence::prelude::*;
use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
use std::net::UdpSocket;

fn main() {
    let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = UdpMetricSink::from(("localhost", DEFAULT_PORT), sock).unwrap();
    let metrics = StatsdClient::from_sink("example.prefix", sink);

    metrics.count("example.counter", 1).unwrap();
    metrics.gauge("example.gauge", 5).unwrap();
    metrics.time("example.timer", 32).unwrap();
    metrics.histogram("example.histogram", 22).unwrap();
    metrics.meter("example.meter", 8).unwrap();
}

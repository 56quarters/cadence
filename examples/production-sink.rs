// Cadence - An extensible Statsd client for Rust!
//
// This file is dual-licensed to the public domain and under the following
// license: you are granted a perpetual, irrevocable license to copy, modify,
// publish, and distribute this file as you see fit.

// This example shows how you might configure the Cadence client for maximum
// isolation and performance. The buffered UDP sink accumulates multiple metrics
// in a buffer before writing to the network. The queuing sink runs the wrapped
// sink in a separate thread ensuring it doesn't interfere with your application.

extern crate cadence;

use cadence::prelude::*;
use cadence::{BufferedUdpMetricSink, QueuingMetricSink, StatsdClient, DEFAULT_PORT};
use std::net::UdpSocket;

fn main() {
    let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
    let buffered = BufferedUdpMetricSink::from(("localhost", DEFAULT_PORT), sock).unwrap();
    let queued = QueuingMetricSink::from(buffered);
    let metrics = StatsdClient::from_sink("example.prefix", queued);

    metrics.count("example.counter", 1).unwrap();
    metrics.gauge("example.gauge", 5).unwrap();
    metrics.time("example.timer", 32).unwrap();
    metrics.histogram("example.histogram", 22).unwrap();
    metrics.meter("example.meter", 8).unwrap();
}

// Cadence - An extensible Statsd client for Rust!
//
// This file is dual-licensed to the public domain and under the following
// license: you are granted a perpetual, irrevocable license to copy, modify,
// publish, and distribute this file as you see fit.

// This example shows how the Cadence client could be used with a 'no-op' sink
// that just discards all metrics. This might be useful if you want to disable
// metric collection for some reason.

extern crate cadence;

use cadence::prelude::*;
use cadence::{StatsdClient, NopMetricSink};


fn main() {
    let sink = NopMetricSink;
    let metrics = StatsdClient::from_sink("example.prefix", sink);

    metrics.count("example.counter", 1).unwrap();
    metrics.gauge("example.gauge", 5).unwrap();
    metrics.time("example.timer", 32).unwrap();
    metrics.histogram("example.histogram", 22).unwrap();
    metrics.meter("example.meter", 8).unwrap();
}

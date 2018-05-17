// Cadence - An extensible Statsd client for Rust!
//
// This file is dual-licensed to the public domain and under the following
// license: you are granted a perpetual, irrevocable license to copy, modify,
// publish, and distribute this file as you see fit.

// This example shows how you might use the Cadence client in your multithreaded
// application by simply cloning a copy of it for each thread that needs to emit
// metrics. This is probably not as performant as using an Arc and doesn't offer
// the advantage of using the trait `MetricClient` instead of the concrete type
// but it's pretty simple.

extern crate cadence;

use cadence::prelude::*;
use cadence::{NopMetricSink, StatsdClient};
use std::thread;

pub trait RequestHandler {
    fn handle(&self) -> Result<(), String>;
}

pub struct ThreadedHandler {
    metrics: StatsdClient,
}

impl ThreadedHandler {
    fn new() -> ThreadedHandler {
        ThreadedHandler {
            metrics: StatsdClient::from_sink("example.prefix", NopMetricSink),
        }
    }
}

impl RequestHandler for ThreadedHandler {
    fn handle(&self) -> Result<(), String> {
        let metrics_copy = self.metrics.clone();

        let t = thread::spawn(move || {
            let _ = metrics_copy.incr("request.handled");
            println!("Hello from a threaded handler!");
        });

        t.join().unwrap();
        Ok(())
    }
}

fn main() {
    let handler = ThreadedHandler::new();
    handler.handle().unwrap();
}

// Cadence - An extensible Statsd client for Rust!
//
// This file is dual-licensed to the public domain and under the following
// license: you are granted a perpetual, irrevocable license to copy, modify,
// publish, and distribute this file as you see fit.

// This example shows how you might use the Cadence client in your multithreaded
// application by wrapping it in an Arc pointer. This allows you to access the
// client from multiple threads.

extern crate cadence;

use std::sync::Arc;
use std::thread;
use cadence::prelude::*;
use cadence::{StatsdClient, NopMetricSink};


pub trait RequestHandler {
    fn handle(&self) -> Result<(), String>;
}

pub struct ThreadedHandler {
    metrics: Arc<MetricClient + Send + Sync>,
}

impl ThreadedHandler {
    fn new() -> ThreadedHandler {
        ThreadedHandler {
            metrics: Arc::new(StatsdClient::from_sink("example.prefix", NopMetricSink)),
        }
    }
}

impl RequestHandler for ThreadedHandler {
    fn handle(&self) -> Result<(), String> {
        let metrics_ref = self.metrics.clone();

        let t = thread::spawn(move || {
            let _ = metrics_ref.incr("request.handled");
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

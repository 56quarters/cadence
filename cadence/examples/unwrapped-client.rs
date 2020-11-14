// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows how you might use the Cadence client in your multithreaded
// application by simply cloning a copy of it for each thread that needs to emit
// metrics. This is probably not as performant as using an Arc and doesn't offer
// the advantage of using the trait `MetricClient` instead of the concrete type
// but it's pretty simple.

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
            let _ = metrics_copy.count("request.handled", 1);
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

// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows how you might use the Cadence client in your multithreaded
// application by wrapping it in an Arc pointer. This allows you to access the
// client from multiple threads.

use cadence::prelude::*;
use cadence::{NopMetricSink, StatsdClient};
use std::sync::Arc;
use std::thread;

pub trait RequestHandler {
    fn handle(&self) -> Result<(), String>;
}

pub struct ThreadedHandler {
    metrics: Arc<dyn MetricClient + Send + Sync>,
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
        let metrics_ref = Arc::clone(&self.metrics);

        let t = thread::spawn(move || {
            let _ = metrics_ref.count("request.handled", 1);
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

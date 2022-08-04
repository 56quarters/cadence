// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

use std::io;
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use std::time::Duration;

use cadence::prelude::*;
use cadence::{MetricSink, NopMetricSink, StatsdClient};

// This example shows how you might use a `MetricSink` with Cadence while
// retaining a reference to it in order to periodically call it's `flush`
// method.

/// MetricSink implementation that delegates to another referenced counted
/// implementation.
#[derive(Clone)]
pub struct CloneableSink {
    wrapped: Arc<dyn MetricSink + Send + Sync + RefUnwindSafe + 'static>,
}

impl CloneableSink {
    pub fn new<T>(wrapped: T) -> Self
    where
        T: MetricSink + Send + Sync + RefUnwindSafe + 'static,
    {
        Self {
            wrapped: Arc::new(wrapped),
        }
    }
}

impl MetricSink for CloneableSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.wrapped.emit(metric)
    }

    fn flush(&self) -> io::Result<()> {
        self.wrapped.flush()
    }
}

fn main() {
    let real_sink = NopMetricSink;
    let reference1 = CloneableSink::new(real_sink);
    let reference2 = reference1.clone();
    let client = StatsdClient::from_sink("prefix", reference1);

    let _ = reference2.flush();

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

    let _ = reference2.flush();
}

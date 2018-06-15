// Cadence - An extensible Statsd client for Rust!
//
// This file is dual-licensed to the public domain and under the following
// license: you are granted a perpetual, irrevocable license to copy, modify,
// publish, and distribute this file as you see fit.

// This example shows how you might create a wrapper `MetricClient` implementation
// around a `StatsdClient` instance that automatically adds tags to all metrics that
// are emitted by it. This wrapper can "stack" using the `::with_tags()` method and
// create new instances that emit all metrics with the tags of the parent in addition
// to its own.

extern crate cadence;

use cadence::prelude::*;
use cadence::{
    Counted, Counter, Gauge, Gauged, Histogram, Histogrammed, Meter, Metered, MetricBuilder,
    MetricSink, Set, Setted, StatsdClient, Timed, Timer,
};
use std::fmt;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Decorator around a `MetricClient` implementation that will always add certain
/// tags to all metrics emitted by the client.
///
/// Decorators can be 'stacked' by calling the `.with_tags()` method to add extra
/// tags to be emitted by a new instance of the decorator. All decorators created
/// via this method will emit the tags provided *in addition* to any tags that were
/// emitted by the original decorator.
#[derive(Clone)]
pub struct MetricTagDecorator {
    client: Arc<MetricClient + Send + Sync>,
    tags: Vec<(String, String)>,
}

impl MetricTagDecorator {
    /// Create a new decorator from the provided client and tags.
    pub fn from_tags_string(
        client: Arc<MetricClient + Send + Sync>,
        tags: Vec<(String, String)>,
    ) -> Self {
        MetricTagDecorator {
            client: client,
            tags: tags,
        }
    }

    /// Create a new decorator from the provided client and tags.
    pub fn from_tags_str(client: Arc<MetricClient + Send + Sync>, tags: Vec<(&str, &str)>) -> Self {
        Self::from_tags_string(client, Self::to_vec_strings(tags.iter()))
    }

    /// Create a new decorator from the provided client and tags.
    pub fn from_tags_slice(client: Arc<MetricClient + Send + Sync>, tags: &[(&str, &str)]) -> Self {
        Self::from_tags_string(client, Self::to_vec_strings(tags.iter()))
    }

    fn to_vec_strings<'a, I>(tags: I) -> Vec<(String, String)>
    where
        I: Iterator<Item = &'a (&'a str, &'a str)>,
    {
        tags.map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    /// Create a new decorator wrapping the current decorator with the provided tags.
    pub fn with_tags_string(&self, mut tags: Vec<(String, String)>) -> Self {
        tags.extend_from_slice(&self.tags);
        Self::from_tags_string(Arc::clone(&self.client), tags)
    }

    /// Create a new decorator wrapping the current decorator with the provided tags.
    pub fn with_tags_str(&self, tags: Vec<(&str, &str)>) -> Self {
        let mut tags = Self::to_vec_strings(tags.iter());
        tags.extend_from_slice(&self.tags);
        Self::from_tags_string(Arc::clone(&self.client), tags)
    }
}

impl Counted for MetricTagDecorator {
    fn count_with_tags<'a>(&'a self, key: &'a str, count: i64) -> MetricBuilder<Counter> {
        let mut builder = self.client.count_with_tags(key, count);
        for (tkey, tval) in self.tags.iter() {
            builder = builder.with_tag(tkey, tval);
        }

        builder
    }
}

impl Timed for MetricTagDecorator {
    fn time_with_tags<'a>(&'a self, key: &'a str, time: u64) -> MetricBuilder<Timer> {
        let mut builder = self.client.time_with_tags(key, time);
        for (tkey, tval) in self.tags.iter() {
            builder = builder.with_tag(tkey, tval);
        }

        builder
    }

    fn time_duration_with_tags<'a>(
        &'a self,
        key: &'a str,
        duration: Duration,
    ) -> MetricBuilder<Timer> {
        let mut builder = self.client.time_duration_with_tags(key, duration);
        for (tkey, tval) in self.tags.iter() {
            builder = builder.with_tag(tkey, tval);
        }

        builder
    }
}

impl Gauged for MetricTagDecorator {
    fn gauge_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Gauge> {
        let mut builder = self.client.gauge_with_tags(key, value);
        for (tkey, tval) in self.tags.iter() {
            builder = builder.with_tag(tkey, tval);
        }

        builder
    }
}

impl Metered for MetricTagDecorator {
    fn meter_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Meter> {
        let mut builder = self.client.meter_with_tags(key, value);
        for (tkey, tval) in self.tags.iter() {
            builder = builder.with_tag(tkey, tval);
        }

        builder
    }
}

impl Histogrammed for MetricTagDecorator {
    fn histogram_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Histogram> {
        let mut builder = self.client.histogram_with_tags(key, value);
        for (tkey, tval) in self.tags.iter() {
            builder = builder.with_tag(tkey, tval);
        }

        builder
    }
}

impl Setted for MetricTagDecorator {
    fn set_with_tags<'a>(&'a self, key: &'a str, value: i64) -> MetricBuilder<Set> {
        let mut builder = self.client.set_with_tags(key, value);
        for (tkey, tval) in self.tags.iter() {
            builder = builder.with_tag(tkey, tval);
        }

        builder
    }
}

impl MetricClient for MetricTagDecorator {}

impl fmt::Debug for MetricTagDecorator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MetricTagDecorator {{ client: ..., tags: {:?} }}", self.tags)
    }
}

struct PrintingSink;

impl MetricSink for PrintingSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        println!("{}", metric);
        Ok(0)
    }
}

fn main() {
    let sink = PrintingSink;
    let client = StatsdClient::from_sink("some.prefix", sink);

    let view1 = MetricTagDecorator::from_tags_str(
        Arc::new(client),
        vec![("host", "a"), ("region", "us-east")],
    );

    // All metrics emitted by `view1` will contain the 'host' and 'region' tags
    view1.incr("some.event").unwrap();
    view1.incr("some.error").unwrap();

    // All metrics emitted in by views in the threads below will contain their
    // thread ID as a tag in the metrics emitted in addition to the tags added
    // above.
    let threads = AtomicUsize::new(1);

    for _ in 0..3 {
        // Increment the counter to indicate we're going to run this next step in
        // a unique thread. Next, create a new decorator for metrics emitted from
        // that thread that includes the thread ID as a tag for those metrics.
        let thread_id = threads.fetch_add(1, Ordering::Acquire);
        let worker_metrics = view1.with_tags_string(
            vec![("thread".to_string(), thread_id.to_string())]
        );

        thread::spawn(move || {
            worker_metrics.incr("some.other.event").unwrap();
            worker_metrics.incr("some.other.error").unwrap();
        }).join().unwrap();
    }
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2019 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;

/// Trait for various backends that send Statsd metrics somewhere.
///
/// The metric string will be in the canonical format to be sent to a
/// Statsd server. The metric string will not include a trailing newline.
/// Examples of each supported metric type are given below.
///
/// ## Counter
///
/// ``` text
/// some.counter:123|c
/// ```
///
/// ## Timer
///
/// ``` text
/// some.timer:456|ms
/// ```
///
/// ## Gauge
///
/// ``` text
/// some.gauge:5|g
/// ```
///
/// ## Meter
///
/// ``` text
/// some.meter:8|m
/// ```
///
/// ## Histogram
///
/// ``` text
/// some.histogram:4|h
/// ```
///
/// ## Set
///
/// ``` text
/// some.set:2|s
/// ```
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait MetricSink {
    /// Send the Statsd metric using this sink and return the number of bytes
    /// written or an I/O error.
    fn emit(&self, metric: &str) -> io::Result<usize>;
}

/// Implementation of a `MetricSink` that discards all metrics.
///
/// Useful for disabling metric collection or unit tests.
#[derive(Debug, Clone)]
pub struct NopMetricSink;

impl MetricSink for NopMetricSink {
    #[allow(unused_variables)]
    fn emit(&self, metric: &str) -> io::Result<usize> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{MetricSink, NopMetricSink};
    #[test]
    fn test_nop_metric_sink() {
        let sink = NopMetricSink;
        assert_eq!(0, sink.emit("baz:4|c").unwrap());
    }
}

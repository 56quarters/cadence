#![allow(deprecated)]

use cadence::prelude::*;
use cadence::{Gauge, Histogram, Meter, NopMetricSink, StatsdClient, Timer};
use std::time::Duration;

fn new_nop_client(prefix: &str) -> StatsdClient {
    StatsdClient::from_sink(prefix, NopMetricSink)
}

#[test]
fn test_statsd_client_time_duration() {
    let client = new_nop_client("client.test");
    let expected = Timer::new("client.test.", "timer.key", 35);
    assert_eq!(
        expected,
        client.time_duration("timer.key", Duration::from_millis(35)).unwrap()
    );
}

#[test]
fn test_statsd_client_gauge_f64() {
    let client = new_nop_client("client.test");
    let expected = Gauge::new_f64("client.test.", "gauge.key", 4.5);
    assert_eq!(expected, client.gauge_f64("gauge.key", 4.5).unwrap());
}

#[test]
fn test_statsd_client_mark() {
    let client = new_nop_client("client.test");
    let expected = Meter::new("client.test.", "meter.key", 1);
    assert_eq!(expected, client.mark("meter.key").unwrap());
}

#[test]
fn test_statsd_client_histogram_duration() {
    let client = new_nop_client("client.test");
    let expected = Histogram::new("client.test.", "histogram.key", 2000);
    assert_eq!(
        expected,
        client
            .histogram_duration("histogram.key", Duration::from_nanos(2000))
            .unwrap()
    );
}

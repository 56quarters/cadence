use cadence::prelude::*;
use cadence::{Counter, Gauge, Histogram, Meter, NopMetricSink, StatsdClient, Timer};
use std::time::Duration;
use utils::run_arc_threaded_test;

mod utils;

fn new_nop_client(prefix: &str) -> StatsdClient {
    StatsdClient::from_sink(prefix, NopMetricSink)
}

#[test]
fn test_statsd_client_count() {
    let client = new_nop_client("client.test");
    let expected = Counter::new("client.test.", "counter.key", 42);
    assert_eq!(expected, client.count("counter.key", 42).unwrap());
}

#[test]
fn test_statsd_client_time() {
    let client = new_nop_client("client.test");
    let expected = Timer::new("client.test.", "timer.key", 25);
    assert_eq!(expected, client.time("timer.key", 25).unwrap());
}

#[test]
fn test_statsd_client_time_duration() {
    let client = new_nop_client("client.test");
    let expected = Timer::new("client.test.", "timer.key", 35);
    assert_eq!(expected, client.time("timer.key", Duration::from_millis(35)).unwrap());
}

#[test]
fn test_statsd_client_gauge() {
    let client = new_nop_client("client.test");
    let expected = Gauge::new("client.test.", "gauge.key", 5);
    assert_eq!(expected, client.gauge("gauge.key", 5).unwrap());
}

#[test]
fn test_statsd_client_gauge_f64() {
    let client = new_nop_client("client.test");
    let expected = Gauge::new_f64("client.test.", "gauge.key", 5.5);
    assert_eq!(expected, client.gauge("gauge.key", 5.5).unwrap());
}

#[test]
fn test_statsd_client_meter() {
    let client = new_nop_client("client.test");
    let expected = Meter::new("client.test.", "meter.key", 7);
    assert_eq!(expected, client.meter("meter.key", 7).unwrap());
}

#[test]
fn test_statsd_client_histogram() {
    let client = new_nop_client("client.test");
    let expected = Histogram::new("client.test.", "histogram.key", 20);
    assert_eq!(expected, client.histogram("histogram.key", 20).unwrap());
}

#[test]
fn test_statsd_client_nop_sink_single_threaded() {
    let client = new_nop_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

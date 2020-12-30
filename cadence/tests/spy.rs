use cadence::{BufferedSpyMetricSink, SpyMetricSink, StatsdClient};
use std::sync::{Arc, Mutex};

mod utils;
use utils::{run_arc_threaded_test, NUM_ITERATIONS, NUM_THREADS};

fn new_spy_client(prefix: &str) -> StatsdClient {
    let writer = Arc::new(Mutex::new(Vec::new()));
    let sink = SpyMetricSink::from(writer);
    StatsdClient::from_sink(prefix, sink)
}

fn new_buffered_spy_client(prefix: &str) -> StatsdClient {
    let writer = Arc::new(Mutex::new(Vec::new()));
    let sink = BufferedSpyMetricSink::from(writer);
    StatsdClient::from_sink(prefix, sink)
}

#[test]
fn test_statsd_client_spy_sink_single_threaded() {
    let client = new_spy_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

#[test]
fn test_statsd_client_buffered_spy_sink_single_threaded() {
    let client = new_buffered_spy_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

#[ignore]
#[test]
fn test_statsd_client_spy_sink_many_threaded() {
    let client = new_spy_client("cadence");
    run_arc_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
}

#[ignore]
#[test]
fn test_statsd_client_buffered_spy_sink_many_threaded() {
    let client = new_buffered_spy_client("cadence");
    run_arc_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
}

#![cfg(unix)]

use cadence::test::UnixServerHarness;
use cadence::{BufferedUnixMetricSink, QueuingMetricSink, StatsdClient, UnixMetricSink};
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use utils::run_arc_threaded_test;

mod utils;

fn new_unix_client<P>(prefix: &str, path: P) -> StatsdClient
where
    P: AsRef<Path>,
{
    let socket = UnixDatagram::unbound().unwrap();
    let sink = UnixMetricSink::from(path, socket);
    StatsdClient::from_sink(prefix, sink)
}

fn new_buffered_unix_client<P>(prefix: &str, path: P) -> StatsdClient
where
    P: AsRef<Path>,
{
    let socket = UnixDatagram::unbound().unwrap();
    let sink = BufferedUnixMetricSink::from(path, socket);
    StatsdClient::from_sink(prefix, sink)
}

fn new_queuing_buffered_unix_client<P>(prefix: &str, path: P) -> StatsdClient
where
    P: AsRef<Path>,
{
    let socket = UnixDatagram::unbound().unwrap();
    let unix = UnixMetricSink::from(path, socket);
    let sink = QueuingMetricSink::from(unix);
    StatsdClient::from_sink(prefix, sink)
}

#[test]
fn test_statsd_client_unix_sink_single_threaded() {
    let harness = UnixServerHarness::new("test_statsd_client_unix_sink_single_threaded");
    harness.run_quiet(|socket| {
        let client = new_unix_client("client.test", socket);
        run_arc_threaded_test(client, 1, 1, None);
    });
}

#[test]
fn test_statsd_client_buffered_unix_sink_single_threaded() {
    let harness = UnixServerHarness::new("test_statsd_client_buffered_unix_sink_single_threaded");
    harness.run_quiet(|socket| {
        let client = new_buffered_unix_client("client.test", socket);
        run_arc_threaded_test(client, 1, 1, None);
    });
}

#[test]
fn test_statsd_client_queuing_buffered_unix_sink_single_threaded() {
    let harness = UnixServerHarness::new("test_statsd_client_queuing_buffered_unix_sink_single_threaded");
    harness.run_quiet(|socket| {
        let client = new_queuing_buffered_unix_client("client.test", socket);
        run_arc_threaded_test(client, 1, 1, None);
    });
}

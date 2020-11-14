use cadence::{BufferedSpyMetricSink, SpyMetricSink, StatsdClient};

mod utils;
use crossbeam_channel::Receiver;
use utils::run_arc_threaded_test;

fn new_spy_client(prefix: &str) -> (Receiver<Vec<u8>>, StatsdClient) {
    let (rx, sink) = SpyMetricSink::new();
    (rx, StatsdClient::from_sink(prefix, sink))
}

fn new_buffered_spy_client(prefix: &str) -> (Receiver<Vec<u8>>, StatsdClient) {
    let (rx, sink) = BufferedSpyMetricSink::new();
    (rx, StatsdClient::from_sink(prefix, sink))
}

#[test]
fn test_statsd_client_spy_sink_single_threaded() {
    let (_rx, client) = new_spy_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

#[test]
fn test_statsd_client_buffered_spy_sink_single_threaded() {
    let (_rx, client) = new_buffered_spy_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

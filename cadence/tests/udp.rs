use cadence::{BufferedUdpMetricSink, QueuingMetricSink, StatsdClient, UdpMetricSink, DEFAULT_PORT};
use std::net::UdpSocket;

mod utils;
use utils::run_arc_threaded_test;

const TARGET_HOST: (&str, u16) = ("127.0.0.1", DEFAULT_PORT);

fn new_udp_client(prefix: &str) -> StatsdClient {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = UdpMetricSink::from(TARGET_HOST, socket).unwrap();
    StatsdClient::from_sink(prefix, sink)
}

fn new_buffered_udp_client(prefix: &str) -> StatsdClient {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::from(TARGET_HOST, socket).unwrap();
    StatsdClient::from_sink(prefix, sink)
}

fn new_queuing_buffered_udp_client(prefix: &str) -> StatsdClient {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let buffered = BufferedUdpMetricSink::from(TARGET_HOST, socket).unwrap();
    let sink = QueuingMetricSink::from(buffered);
    StatsdClient::from_sink(prefix, sink)
}

#[test]
fn test_statsd_client_udp_sink_single_threaded() {
    let client = new_udp_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

#[test]
fn test_statsd_client_buffered_udp_sink_single_threaded() {
    let client = new_buffered_udp_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

#[test]
fn test_statsd_client_queuing_buffered_udp_sink_single_threaded() {
    let client = new_queuing_buffered_udp_client("cadence");
    run_arc_threaded_test(client, 1, 1);
}

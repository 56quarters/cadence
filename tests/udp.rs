use cadence::test::DelegatingMetricSink;
use cadence::{
    BufferedUdpMetricSink, QueuingMetricSink, StatsdClient, UdpMetricSink, DEFAULT_PORT,
};
use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

mod utils;
use utils::{run_arc_threaded_test, NUM_ITERATIONS, NUM_THREADS};

const TARGET_HOST: (&str, u16) = ("127.0.0.1", DEFAULT_PORT);
const BUFFER_SZ: usize = 512;
const QUEUE_SZ: usize = 512 * 1024;

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

fn new_delegating_queuing_buffered_udp_client(prefix: &str) -> (StatsdClient, Arc<QueuingMetricSink>) {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let buffered = BufferedUdpMetricSink::with_capacity(TARGET_HOST, socket, BUFFER_SZ).unwrap();
    let queuing = Arc::new(QueuingMetricSink::with_capacity(buffered, QUEUE_SZ));
    let sink = DelegatingMetricSink::new(queuing.clone());
    let client = StatsdClient::from_sink(prefix, sink);
    (client, queuing)
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

#[ignore]
#[test]
fn test_statsd_client_udp_sink_many_threaded() {
    let client = new_udp_client("cadence");
    run_arc_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
}

#[ignore]
#[test]
fn test_statsd_client_buffered_udp_sink_many_threaded() {
    let client = new_buffered_udp_client("cadence");
    run_arc_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
}

#[ignore]
#[test]
fn test_statsd_client_queuing_buffered_udp_sink_many_threaded() {
    let client = new_queuing_buffered_udp_client("cadence");
    run_arc_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
}

#[ignore]
#[test]
fn test_statsd_client_queuing_delegating_many_threaded() {
    let (client, sink) = new_delegating_queuing_buffered_udp_client("cadence");
    run_arc_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);

    let mut queued = sink.queued();
    while queued > 0 {
        queued = sink.queued();
        thread::yield_now();
    }
}

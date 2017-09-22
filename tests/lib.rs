extern crate cadence;

use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::sync::Arc;

use cadence::prelude::*;
use cadence::{DEFAULT_PORT, NopMetricSink, BufferedUdpMetricSink,
              StatsdClient, QueuingMetricSink, Counter, Timer, Gauge,
              Meter, Histogram};


fn new_nop_client(prefix: &str) -> StatsdClient {
    StatsdClient::from_sink(prefix, NopMetricSink)
}


fn new_udp_client(prefix: &str) -> StatsdClient {
    let addr = ("127.0.0.1", DEFAULT_PORT);
    StatsdClient::from_udp_host(prefix, addr).unwrap()
}


fn new_buffered_udp_client(prefix: &str) -> StatsdClient {
    let host = ("127.0.0.1", DEFAULT_PORT);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    StatsdClient::from_sink(prefix, sink)
}


fn new_queuing_buffered_udp_client(prefix: &str) -> StatsdClient {
    let host = ("127.0.0.1", DEFAULT_PORT);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    StatsdClient::from_sink(prefix, QueuingMetricSink::from(sink))
}


#[test]
fn test_statsd_client_incr() {
    let client = new_nop_client("client.test");
    let expected = Counter::new("client.test", "counter.key", 1);
    assert_eq!(expected, client.incr("counter.key").unwrap());
}


#[test]
fn test_statsd_client_decr() {
    let client = new_nop_client("client.test");
    let expected = Counter::new("client.test", "counter.key", -1);
    assert_eq!(expected, client.decr("counter.key").unwrap());
}


#[test]
fn test_statsd_client_count() {
    let client = new_nop_client("client.test");
    let expected = Counter::new("client.test", "counter.key", 42);
    assert_eq!(expected, client.count("counter.key", 42).unwrap());
}


#[test]
fn test_statsd_client_time() {
    let client = new_nop_client("client.test");
    let expected = Timer::new("client.test", "timer.key", 25);
    assert_eq!(expected, client.time("timer.key", 25).unwrap());
}


#[test]
fn test_statsd_client_time_duration() {
    let client = new_nop_client("client.test");
    let expected = Timer::new("client.test", "timer.key", 35);
    assert_eq!(expected, client.time_duration("timer.key", Duration::from_millis(35)).unwrap());
}


#[test]
fn test_statsd_client_gauge() {
    let client = new_nop_client("client.test");
    let expected = Gauge::new("client.test", "gauge.key", 5);
    assert_eq!(expected, client.gauge("gauge.key", 5).unwrap());
}


#[test]
fn test_statsd_client_mark() {
    let client = new_nop_client("client.test");
    let expected = Meter::new("client.test", "meter.key", 1);
    assert_eq!(expected, client.mark("meter.key").unwrap());
}


#[test]
fn test_statsd_client_meter() {
    let client = new_nop_client("client.test");
    let expected = Meter::new("client.test", "meter.key", 7);
    assert_eq!(expected, client.meter("meter.key", 7).unwrap());
}


#[test]
fn test_statsd_client_histogram() {
    let client = new_nop_client("client.test");
    let expected = Histogram::new("client.test", "histogram.key", 20);
    assert_eq!(expected, client.histogram("histogram.key", 20).unwrap());
}


#[test]
fn test_statsd_client_nop_sink_single_threaded() {
    let client = new_nop_client("counter.threaded.nop");
    run_arc_threaded_test(client, 1, 1);
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


const NUM_THREADS: u64 = 100;
const NUM_ITERATIONS: u64 = 1_000;


#[ignore]
#[test]
fn test_statsd_client_nop_sink_many_threaded() {
    let client = new_nop_client("cadence");
    run_arc_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
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


fn run_arc_threaded_test(client: StatsdClient, num_threads: u64, iterations: u64) {
    let shared_client = Arc::new(client);

    let threads: Vec<_> = (0..num_threads).map(|_| {
        let local_client = Arc::clone(&shared_client);

        thread::spawn(move || {
            for i in 0..iterations {
                local_client.count("some.counter", i as i64).unwrap();
                local_client.time("some.timer", i).unwrap();
                local_client.gauge("some.gauge", i).unwrap();
                local_client.meter("some.meter", i).unwrap();
                local_client.histogram("some.histogram", i).unwrap();
                thread::sleep(Duration::from_millis(1));
            }
        })
    }).collect();

    for t in threads {
        t.join().unwrap();
    }
}

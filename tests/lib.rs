//!
//!
//!

#![feature(test)]

extern crate test;
extern crate statsd;

use std::thread;
use std::sync::Arc;

use statsd::{
    DEFAULT_PORT,
    NopMetricSink,
    UdpMetricSink,
    MetricSink,
    StatsdClient,
    Counted,
    Timed,
    Gauged,
    Metered,
    Counter,
    Timer,
    Gauge,
    Meter
};


fn new_nop_client(prefix: &str) -> StatsdClient<NopMetricSink> {
    StatsdClient::from_sink(prefix, NopMetricSink)
}


fn new_udp_client(prefix: &str) -> StatsdClient<UdpMetricSink> {
    let addr = ("127.0.0.1", DEFAULT_PORT);
    StatsdClient::<UdpMetricSink>::from_udp_host(prefix, addr).unwrap()
}



#[test]
fn test_statsd_client_incr() {
    let client = new_nop_client("client.test");
    let expected = Counter::new("client.test.counter.key", 1);
    assert_eq!(expected, client.incr("counter.key").unwrap());
}


#[test]
fn test_statsd_client_decr() {
    let client = new_nop_client("client.test");
    let expected = Counter::new("client.test.counter.key", -1);
    assert_eq!(expected, client.decr("counter.key").unwrap());
}


#[test]
fn test_statsd_client_count() {
    let client = new_nop_client("client.test");
    let expected = Counter::new("client.test.counter.key", 42);
    assert_eq!(expected, client.count("counter.key", 42).unwrap());
}


#[test]
fn test_statsd_client_time() {
    let client = new_nop_client("client.test");
    let expected = Timer::new("client.test.timer.key", 25);
    assert_eq!(expected, client.time("timer.key", 25).unwrap());
}


#[test]
fn test_statsd_client_gauge() {
    let client = new_nop_client("client.test");
    let expected = Gauge::new("client.test.gauge.key", 5);
    assert_eq!(expected, client.gauge("gauge.key", 5).unwrap());
}


#[test]
fn test_statsd_client_mark() {
    let client = new_nop_client("client.test");
    let expected = Meter::new("client.test.meter.key", 1);
    assert_eq!(expected, client.mark("meter.key").unwrap());
}


#[test]
fn test_statsd_client_meter() {
    let client = new_nop_client("client.test");
    let expected = Meter::new("client.test.meter.key", 7);
    assert_eq!(expected, client.meter("meter.key", 7).unwrap());
}


#[ignore]
#[test]
fn test_statsd_client_nop_sink_single_threaded() {
    let client = new_nop_client("counter.threaded.nop");
    run_threaded_test(client, 1, 1);
}


#[ignore]
#[test]
fn test_statsd_client_udp_sink_single_threaded() {
    let client = new_udp_client("counter.threaded.udp");
    run_threaded_test(client, 1, 1);
}


const NUM_THREADS: u64 = 100;
const NUM_ITERATIONS: u64 = 10_000;


#[ignore]
#[test]
fn test_statsd_client_nop_sink_many_threaded() {
    let client = new_nop_client("counter.threaded.nop");
    run_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
}


#[ignore]
#[test]
fn test_statsd_client_udp_sink_many_threaded() {
    let client = new_udp_client("counter.threaded.udp");
    run_threaded_test(client, NUM_THREADS, NUM_ITERATIONS);
}


fn run_threaded_test<T>(client: StatsdClient<T>, num_threads: u64, iterations: u64) -> ()
    where T: 'static + MetricSink + Sync + Send {
    let shared_client = Arc::new(client);

    let threads: Vec<_> = (0..num_threads).map(|_| {
        let local_client = shared_client.clone();
        
        thread::spawn(move || {
            for i in 0..iterations {
                local_client.count("some.metric", i as i64).unwrap();
            }
        })
    }).collect();

    for t in threads {
        t.join().unwrap();
    }
}

//!
//!
//!

#![feature(test)]

extern crate test;
extern crate statsd;

use std::net::UdpSocket;
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
    Metered
};


struct CounterHolder<'a, T: Counted + 'a> {
    counter: &'a T
}


struct TimerHolder<'a, T: Timed + 'a> {
    timer: &'a T
}


struct GaugeHolder<'a, T: Gauged + 'a> {
    gauge: &'a T
}


struct MeterHolder<'a, T: Metered + 'a> {
    meter: &'a T
}


fn new_nop_client(prefix: &str) -> StatsdClient<NopMetricSink> {
    let sink = NopMetricSink;
    StatsdClient::new(prefix, sink)
}


fn new_udp_client(prefix: &str) -> StatsdClient<UdpMetricSink<(&str, u16)>> {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = UdpMetricSink::new(("127.0.0.1", DEFAULT_PORT), socket);
    StatsdClient::new(prefix, sink)
}


#[test]
fn test_statsd_client_as_counter() {
    let client = new_nop_client("counter.test");
    let holder = CounterHolder{counter: &client};

    holder.counter.sample("some.counter.metric", 13, 0.1).unwrap();
}


#[test]
fn test_statsd_client_as_timer() {
    let client = new_nop_client("timer.test");
    let holder = TimerHolder{timer: &client};

    holder.timer.time("some.timer.metric", 25).unwrap();
}


#[test]
fn test_statsd_client_as_gauge() {
    let client = new_nop_client("gauge.test");
    let holder = GaugeHolder{gauge: &client};

    holder.gauge.gauge("some.gauge.metric", 98).unwrap();
}


#[test]
fn test_statsd_client_as_meter() {
    let client = new_nop_client("meter.test");
    let holder = MeterHolder{meter: &client};

    holder.meter.meter("some.meter.metric", 5).unwrap();
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

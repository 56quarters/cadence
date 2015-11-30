//!
//!
//!

#![feature(test)]

extern crate test;
extern crate statsd;

use std::net::UdpSocket;
use std::thread;
use std::sync::Arc;

use statsd::client::{
    DEFAULT_PORT,
    NopMetricSink,
    UdpMetricSink,
    MetricSink,
    StatsdClient,
    Counted,
    Timed,
    Gauged
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

    holder.counter.count("some.counter.metric", 13, None).unwrap();
}


#[test]
fn test_statsd_client_as_timer() {
    let client = new_nop_client("timer.test");
    let holder = TimerHolder{timer: &client};

    holder.timer.time("some.timer.metric", 25, None).unwrap();
}


#[test]
fn test_statsd_client_as_gauge() {
    let client = new_nop_client("gauge.test");
    let holder = GaugeHolder{gauge: &client};

    holder.gauge.gauge("some.gauge.metric", 98).unwrap();
}


#[ignore]
#[test]
fn test_statsd_client_nop_sink_single_threaded() {
    let client = new_nop_client("counter.threaded.nop");
    run_threaded_test(client, 1);
}


#[ignore]
#[test]
fn test_statsd_client_udp_sink_single_threaded() {
    let client = new_udp_client("counter.threaded.udp");
    run_threaded_test(client, 1);
}


#[ignore]
#[test]
fn test_statsd_client_nop_sink_many_threaded() {
    let client = new_nop_client("counter.threaded.nop");
    run_threaded_test(client, 1000);
}


#[ignore]
#[test]
fn test_statsd_client_udp_sink_many_threaded() {
    let client = new_udp_client("counter.threaded.udp");
    run_threaded_test(client, 1000);
}


fn run_threaded_test<T>(
    client: StatsdClient<T>, threads: u64) where T: 'static + MetricSink + Sync + Send {
    let shared_client = Arc::new(client);
    
    for i in 0..threads {
        let local_client = shared_client.clone();
        
        thread::spawn(move || {
            local_client.count("some.metric", i, None).unwrap();
        });
    }
}

//!
//!
//!

extern crate statsd;

use std::net::{UdpSocket};

use statsd::client::{
    DEFAULT_PORT,
    UdpMetricSink,
    StatsdClient,
    Counted,
    Timed,
    Gauged
};


const METRIC_HOST: &'static str = "127.0.0.1";
const LOCAL_ADDR: &'static str = "0.0.0.0:0";


struct CounterHolder<'a, T: Counted + 'a> {
    counter: &'a T
}


struct TimerHolder<'a, T: Timed + 'a> {
    timer: &'a T
}


struct GaugeHolder<'a, T: Gauged + 'a> {
    gauge: &'a T
}


fn new_local_client(prefix: &str) -> StatsdClient<UdpMetricSink<(&str, u16)>> {
    let metric_host = (METRIC_HOST, DEFAULT_PORT);
    let socket = UdpSocket::bind(LOCAL_ADDR).unwrap();
    let sink = UdpMetricSink::new(metric_host, socket);
    StatsdClient::new(prefix, sink)
}


#[test]
fn test_statsd_client_as_counter() {
    let client = new_local_client("counter.test");
    let holder = CounterHolder{counter: &client};

    for i in 0..45 {
        holder.counter.count("some.counter.metric", i, None);
    }
}


#[test]
fn test_statsd_client_as_timer() {
    let client = new_local_client("timer.test");
    let holder = TimerHolder{timer: &client};

    for i in 10..35 {
        holder.timer.time("some.timer.metric", i, None);
    }
}


#[test]
fn test_statsd_client_as_gauge() {
    let client = new_local_client("gauge.test");
    let holder = GaugeHolder{gauge: &client};

    for i in 90..100 {
        holder.gauge.gauge("some.gauge.metric", i);
    }
}

//!
//!
//!

#![feature(test)]

extern crate test;
extern crate statsd;

use test::Bencher;

use statsd::client::{
    ConsoleMetricSink,
    NopMetricSink,
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


fn new_console_client(prefix: &str) -> StatsdClient<ConsoleMetricSink> {
    let sink = ConsoleMetricSink;
    StatsdClient::new(prefix, sink)
}


fn new_nop_client(prefix: &str) -> StatsdClient<NopMetricSink> {
    let sink = NopMetricSink;
    StatsdClient::new(prefix, sink)
}


#[test]
fn test_statsd_client_as_counter() {
    let client = new_console_client("counter.test");
    let holder = CounterHolder{counter: &client};

    holder.counter.count("some.counter.metric", 13, None).unwrap();
}


#[bench]
fn test_statsd_client_counter_performance(b: &mut Bencher) {
    let client = new_nop_client("counter.perf");
    b.iter(|| client.count("some.counter.metric", 26, None).unwrap())
}


#[test]
fn test_statsd_client_as_timer() {
    let client = new_console_client("timer.test");
    let holder = TimerHolder{timer: &client};

    holder.timer.time("some.timer.metric", 25, None).unwrap();
}


#[bench]
fn test_statsd_client_timer_performance(b: &mut Bencher) {
    let client = new_nop_client("timer.perf");
    b.iter(|| client.time("some.timer.metric", 50, None).unwrap())
}


#[test]
fn test_statsd_client_as_gauge() {
    let client = new_console_client("gauge.test");
    let holder = GaugeHolder{gauge: &client};

    holder.gauge.gauge("some.gauge.metric", 98).unwrap();
}


#[bench]
fn test_statsd_client_gauge_performance(b: &mut Bencher) {
    let client = new_nop_client("gauge.perf");
    b.iter(|| client.gauge("some.gauge.metric", 98).unwrap())
}

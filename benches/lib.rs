#![feature(test)]
extern crate test;
extern crate cadence;

use test::Bencher;

use cadence::{
    StatsdClient,
    Counted,
    Timed,
    Gauged,
    Metered,
    NopMetricSink
};


#[bench]
fn test_benchmark_statsdclient_counter(b: &mut Bencher) {
    let client = StatsdClient::from_sink("test.bench", NopMetricSink);
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_timer(b: &mut Bencher) {
    let client = StatsdClient::from_sink("test.bench", NopMetricSink);
    b.iter(|| client.time("some.timer", 4));
}


#[bench]
fn test_benchmark_statsdclient_gauge(b: &mut Bencher) {
    let client = StatsdClient::from_sink("test.bench", NopMetricSink);
    b.iter(|| client.gauge("some.gauge", 4));
}


#[bench]
fn test_benchmark_statsdclient_meter(b: &mut Bencher) {
    let client = StatsdClient::from_sink("test.bench", NopMetricSink);
    b.iter(|| client.meter("some.meter", 4));
}


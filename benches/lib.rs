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


fn new_nop_client() -> StatsdClient<NopMetricSink> {
    StatsdClient::from_sink("test.bench", NopMetricSink)
}


#[bench]
fn test_benchmark_statsdclient_count(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_incr(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.incr("some.counter.incr"));
}


#[bench]
fn test_benchmark_statsdclient_decr(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.decr("some.counter.decr"));
}


#[bench]
fn test_benchmark_statsdclient_time(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.time("some.timer", 4));
}


#[bench]
fn test_benchmark_statsdclient_gauge(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.gauge("some.gauge", 4));
}


#[bench]
fn test_benchmark_statsdclient_meter(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.meter("some.meter", 4));
}


#[bench]
fn test_benchmark_statsdclient_mark(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.mark("some.meter.mark"));
}

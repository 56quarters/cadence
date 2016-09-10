#![feature(test)]
extern crate test;
extern crate cadence;

use test::Bencher;

use std::net::UdpSocket;

use cadence::prelude::*;
use cadence::{DEFAULT_PORT, StatsdClient, Counter, Timer, Gauge, Meter,
              NopMetricSink, UdpMetricSink, BufferedUdpMetricSink,
              AsyncMetricSink};


fn new_nop_client() -> StatsdClient<NopMetricSink> {
    StatsdClient::from_sink("client.bench", NopMetricSink)
}


fn new_udp_client() -> StatsdClient<UdpMetricSink> {
    let host = ("127.0.0.1", DEFAULT_PORT);
    StatsdClient::<UdpMetricSink>::from_udp_host(
        "client.bench", host).unwrap()
}


fn new_buffered_udp_client() -> StatsdClient<BufferedUdpMetricSink> {
    let host = ("127.0.0.1", DEFAULT_PORT);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    StatsdClient::from_sink("client.bench", sink)
}


fn new_async_buffered_udp_client() -> StatsdClient<AsyncMetricSink<BufferedUdpMetricSink>> {
    let host = ("127.0.0.1", DEFAULT_PORT);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    let async = AsyncMetricSink::from(sink);
    StatsdClient::from_sink("client.bench", async)
}


#[bench]
fn test_benchmark_statsdclient_count_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_incr_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.incr("some.counter.incr"));
}


#[bench]
fn test_benchmark_statsdclient_decr_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.decr("some.counter.decr"));
}


#[bench]
fn test_benchmark_statsdclient_time_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.time("some.timer", 4));
}


#[bench]
fn test_benchmark_statsdclient_gauge_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.gauge("some.gauge", 4));
}


#[bench]
fn test_benchmark_statsdclient_meter_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.meter("some.meter", 4));
}


#[bench]
fn test_benchmark_statsdclient_mark_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.mark("some.meter.mark"));
}


#[bench]
fn test_benchmark_statsdclient_count_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_incr_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.incr("some.counter.incr"));
}


#[bench]
fn test_benchmark_statsdclient_decr_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.decr("some.counter.decr"));
}


#[bench]
fn test_benchmark_statsdclient_time_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.time("some.timer", 4));
}


#[bench]
fn test_benchmark_statsdclient_gauge_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.gauge("some.gauge", 4));
}


#[bench]
fn test_benchmark_statsdclient_meter_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.meter("some.meter", 4));
}


#[bench]
fn test_benchmark_statsdclient_mark_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.mark("some.meter.mark"));
}


#[bench]
fn test_benchmark_statsdclient_count_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_incr_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.incr("some.counter.incr"));
}


#[bench]
fn test_benchmark_statsdclient_decr_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.decr("some.counter.decr"));
}


#[bench]
fn test_benchmark_statsdclient_time_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.time("some.timer", 4));
}


#[bench]
fn test_benchmark_statsdclient_gauge_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.gauge("some.gauge", 4));
}


#[bench]
fn test_benchmark_statsdclient_meter_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.meter("some.meter", 4));
}


#[bench]
fn test_benchmark_statsdclient_mark_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.mark("some.meter.mark"));
}


#[bench]
fn test_benchmark_statsdclient_count_async_buffered_udp(b: &mut Bencher) {
    let client = new_async_buffered_udp_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_incr_async_buffered_udp(b: &mut Bencher) {
    let client = new_async_buffered_udp_client();
    b.iter(|| client.incr("some.counter.incr"));
}


#[bench]
fn test_benchmark_statsdclient_decr_async_buffered_udp(b: &mut Bencher) {
    let client = new_async_buffered_udp_client();
    b.iter(|| client.decr("some.counter.decr"));
}


#[bench]
fn test_benchmark_statsdclient_time_async_buffered_udp(b: &mut Bencher) {
    let client = new_async_buffered_udp_client();
    b.iter(|| client.time("some.timer", 4));
}


#[bench]
fn test_benchmark_statsdclient_gauge_async_buffered_udp(b: &mut Bencher) {
    let client = new_async_buffered_udp_client();
    b.iter(|| client.gauge("some.gauge", 4));
}


#[bench]
fn test_benchmark_statsdclient_meter_async_buffered_udp(b: &mut Bencher) {
    let client = new_async_buffered_udp_client();
    b.iter(|| client.meter("some.meter", 4));
}


#[bench]
fn test_benchmark_statsdclient_mark_async_buffered_udp(b: &mut Bencher) {
    let client = new_async_buffered_udp_client();
    b.iter(|| client.mark("some.meter.mark"));
}


#[bench]
fn test_benchmark_new_counter_obj(b: &mut Bencher) {
    b.iter(|| Counter::new("prefix", "some.counter", 5));
}

#[bench]
fn test_benchmark_new_timer_obj(b: &mut Bencher) {
    b.iter(|| Timer::new("prefix", "some.timer", 5));
}


#[bench]
fn test_benchmark_new_gauge_obj(b: &mut Bencher) {
    b.iter(|| Gauge::new("prefix", "some.gauge", 5));
}


#[bench]
fn test_benchmark_new_meter_obj(b: &mut Bencher) {
    b.iter(|| Meter::new("prefix", "some.meter", 5));
}

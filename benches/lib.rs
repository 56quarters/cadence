#![feature(test)]
extern crate test;
extern crate cadence;

use test::Bencher;

use std::net::UdpSocket;

use cadence::prelude::*;
use cadence::{DEFAULT_PORT, StatsdClient, Counter, Timer, Gauge, Meter,
              Histogram, NopMetricSink, BufferedUdpMetricSink,
              QueuingMetricSink};


fn new_nop_client() -> StatsdClient {
    StatsdClient::from_sink("client.bench", NopMetricSink)
}


fn new_udp_client() -> StatsdClient {
    let host = ("127.0.0.1", DEFAULT_PORT);
    StatsdClient::from_udp_host("client.bench", host).unwrap()
}


fn new_buffered_udp_client() -> StatsdClient {
    let host = ("127.0.0.1", DEFAULT_PORT);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    StatsdClient::from_sink("client.bench", sink)
}


fn new_queuing_nop_client() -> StatsdClient {
    let async = QueuingMetricSink::from(NopMetricSink);
    StatsdClient::from_sink("client.bench", async)
}


#[bench]
fn test_benchmark_statsdclient_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.count("some.counter", 4));
}


#[bench]
fn test_benchmark_statsdclient_queuing_nop(b: &mut Bencher) {
    let client = new_queuing_nop_client();
    b.iter(|| client.count("some.counter", 4));
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


#[bench]
fn test_benchmark_new_histogram_obj(b: &mut Bencher) {
    b.iter(|| Histogram::new("prefix", "some.histogram", 5));
}

#![feature(test)]

extern crate test;

use test::Bencher;

use std::net::UdpSocket;

use cadence::prelude::*;
use cadence::{
    BufferedUdpMetricSink, Counter, Gauge, Histogram, Meter, NopMetricSink, QueuingMetricSink, Set,
    StatsdClient, Timer, UdpMetricSink, DEFAULT_PORT,
};

const TARGET_HOST: (&str, u16) = ("127.0.0.1", DEFAULT_PORT);

fn new_nop_client() -> StatsdClient {
    StatsdClient::from_sink("client.bench", NopMetricSink)
}

fn new_udp_client() -> StatsdClient {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = UdpMetricSink::from(TARGET_HOST, socket).unwrap();
    StatsdClient::from_sink("client.bench", sink)
}

fn new_buffered_udp_client() -> StatsdClient {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = BufferedUdpMetricSink::from(TARGET_HOST, socket).unwrap();
    StatsdClient::from_sink("client.bench", sink)
}

fn new_queuing_nop_client() -> StatsdClient {
    let queuing = QueuingMetricSink::from(NopMetricSink);
    StatsdClient::from_sink("client.bench", queuing)
}

#[bench]
fn test_benchmark_statsdclient_nop(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| client.count("some.counter", 4));
}

#[bench]
fn test_benchmark_statsdclient_nop_with_tags(b: &mut Bencher) {
    let client = new_nop_client();
    b.iter(|| {
        client
            .count_with_tags("some.counter", 4)
            .with_tag("host", "app21.example.com")
            .with_tag("bucket", "3")
            .send();
    });
}

#[bench]
fn test_benchmark_statsdclient_udp(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| client.count("some.counter", 4));
}

#[bench]
fn test_benchmark_statsdclient_udp_with_tags(b: &mut Bencher) {
    let client = new_udp_client();
    b.iter(|| {
        client
            .count_with_tags("some.counter", 4)
            .with_tag("host", "fs03.example.com")
            .with_tag("version", "123")
            .send();
    });
}

#[bench]
fn test_benchmark_statsdclient_buffered_udp(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| client.count("some.counter", 4));
}

#[bench]
fn test_benchmark_statsdclient_buffered_udp_with_tags(b: &mut Bencher) {
    let client = new_buffered_udp_client();
    b.iter(|| {
        client
            .count_with_tags("some.counter", 4)
            .with_tag("user-type", "authenticated")
            .with_tag("bucket", "42")
            .send();
    });
}

#[bench]
fn test_benchmark_statsdclient_queuing_nop(b: &mut Bencher) {
    let client = new_queuing_nop_client();
    b.iter(|| client.count("some.counter", 4));
}

#[bench]
fn test_benchmark_statsdclient_queuing_nop_with_tags(b: &mut Bencher) {
    let client = new_queuing_nop_client();
    b.iter(|| {
        client
            .count_with_tags("some.counter", 4)
            .with_tag("host", "web32.example.com")
            .with_tag("platform", "ng")
            .send();
    });
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
fn test_benchmark_new_gauge_f64_obj(b: &mut Bencher) {
    b.iter(|| Gauge::new_f64("prefix", "some.gauge", 5.5));
}

#[bench]
fn test_benchmark_new_meter_obj(b: &mut Bencher) {
    b.iter(|| Meter::new("prefix", "some.meter", 5));
}

#[bench]
fn test_benchmark_new_histogram_obj(b: &mut Bencher) {
    b.iter(|| Histogram::new("prefix", "some.histogram", 5));
}

#[bench]
fn test_benchmark_new_set_obj(b: &mut Bencher) {
    b.iter(|| Set::new("prefix", "some.set", 8));
}

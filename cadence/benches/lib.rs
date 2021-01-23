use cadence::prelude::*;
use cadence::{
    BufferedUdpMetricSink, Counter, Gauge, Histogram, Meter, NopMetricSink, QueuingMetricSink, Set, StatsdClient,
    Timer, UdpMetricSink, DEFAULT_PORT,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::net::UdpSocket;

const TARGET_HOST: (&str, u16) = ("127.0.0.1", DEFAULT_PORT);
const QUEUE_SIZE: usize = 512 * 1024;

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

fn new_queuing_nop_client(bound: Option<usize>) -> StatsdClient {
    let queuing = if let Some(v) = bound {
        QueuingMetricSink::with_capacity(NopMetricSink, v)
    } else {
        QueuingMetricSink::from(NopMetricSink)
    };

    StatsdClient::from_sink("client.bench", queuing)
}
fn new_queuing_buffered_udp_client(bound: Option<usize>) -> StatsdClient {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let buffered = BufferedUdpMetricSink::from(TARGET_HOST, socket).unwrap();
    let queuing = if let Some(v) = bound {
        QueuingMetricSink::with_capacity(buffered, v)
    } else {
        QueuingMetricSink::from(buffered)
    };

    StatsdClient::from_sink("client.bench", queuing)
}

fn benchmark_statsdclient_nop(c: &mut Criterion) {
    c.bench_function("statsdclient_nop", |b| {
        let client = new_nop_client();
        b.iter(|| client.count("some.counter", 4));
    });

    c.bench_function("statsdclient_nop_with_tags", |b| {
        let client = new_nop_client();
        b.iter(|| {
            client
                .count_with_tags("some.counter", 4)
                .with_tag("host", "app21.example.com")
                .with_tag("bucket", "3")
                .send();
        });
    });
}

fn benchmark_statsdclient_udp(c: &mut Criterion) {
    c.bench_function("statsdclient_udp", |b| {
        let client = new_udp_client();
        b.iter(|| client.count("some.counter", 4));
    });

    c.bench_function("statsdclient_udp_with_tags", |b| {
        let client = new_udp_client();
        b.iter(|| {
            client
                .count_with_tags("some.counter", 4)
                .with_tag("host", "fs03.example.com")
                .with_tag("version", "123")
                .send();
        })
    });
}

fn benchmark_statsdclient_buffered_udp(c: &mut Criterion) {
    c.bench_function("statsdclient_buffered_udp", |b| {
        let client = new_buffered_udp_client();
        b.iter(|| client.count("some.counter", 4));
    });

    c.bench_function("statsdclient_buffered_udp_with_tags", |b| {
        let client = new_buffered_udp_client();

        b.iter(|| {
            client
                .count_with_tags("some.counter", 4)
                .with_tag("user-type", "authenticated")
                .with_tag("bucket", "42")
                .send();
        })
    });
}

fn benchmark_statsdclient_queuing(c: &mut Criterion) {
    c.bench_function("statsdclient_queuing_nop", |b| {
        let client = new_queuing_nop_client(None);
        b.iter(|| client.count("some.counter", 4));
    });

    c.bench_function("statsdclient_queuing_nop_with_tags", |b| {
        let client = new_queuing_nop_client(None);
        b.iter(|| {
            client
                .count_with_tags("some.counter", 4)
                .with_tag("host", "web32.example.com")
                .with_tag("platform", "ng")
                .send();
        })
    });

    c.bench_function("statsdclient_queuing_buffered_udp", |b| {
        let client = new_queuing_buffered_udp_client(None);
        b.iter(|| client.count("some.counter", 4));
    });

    c.bench_function("statsdclient_queuing_buffered_udp_with_tags", |b| {
        let client = new_queuing_buffered_udp_client(None);
        b.iter(|| {
            client
                .count_with_tags("some.counter", 4)
                .with_tag("host", "web32.example.com")
                .with_tag("platform", "ng")
                .send();
        })
    });

    c.bench_function("statsdclient_queuing_nop_back_pressure", |b| {
        let client = new_queuing_nop_client(Some(QUEUE_SIZE));
        b.iter(|| client.count("some.counter", 4));
    });

    c.bench_function("statsdclient_queuing_nop_with_tags_back_pressure", |b| {
        let client = new_queuing_nop_client(Some(QUEUE_SIZE));
        b.iter(|| {
            client
                .count_with_tags("some.counter", 4)
                .with_tag("host", "web32.example.com")
                .with_tag("platform", "ng")
                .send();
        })
    });

    c.bench_function("statsdclient_queuing_buffered_udp_back_pressure", |b| {
        let client = new_queuing_buffered_udp_client(Some(QUEUE_SIZE));
        b.iter(|| client.count("some.counter", 4));
    });

    c.bench_function("statsdclient_queuing_buffered_udp_with_tags_back_pressure", |b| {
        let client = new_queuing_buffered_udp_client(Some(QUEUE_SIZE));
        b.iter(|| {
            client
                .count_with_tags("some.counter", 4)
                .with_tag("host", "web32.example.com")
                .with_tag("platform", "ng")
                .send();
        })
    });
}

fn benchmark_new_metric_obj(c: &mut Criterion) {
    c.bench_function("counter_new", |b| b.iter(|| Counter::new("prefix", "some.counter", 5)));
    c.bench_function("timer_new", |b| b.iter(|| Timer::new("prefix", "some.timer", 5)));
    c.bench_function("gauge_new", |b| b.iter(|| Gauge::new("prefix", "some.gauge", 5)));
    c.bench_function("gauge_new_f64", |b| {
        b.iter(|| Gauge::new_f64("prefix", "some.gauge", 5.1))
    });
    c.bench_function("meter_new", |b| b.iter(|| Meter::new("prefix", "some.meter", 5)));
    c.bench_function("histogram_new", |b| {
        b.iter(|| Histogram::new("prefix", "some.histogram", 5))
    });
    c.bench_function("set_new", |b| b.iter(|| Set::new("prefix", "some.set", 8)));
}

criterion_group!(
    benches,
    benchmark_statsdclient_nop,
    benchmark_statsdclient_udp,
    benchmark_statsdclient_buffered_udp,
    benchmark_statsdclient_queuing,
    benchmark_new_metric_obj
);

criterion_main!(benches);

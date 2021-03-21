use cadence::{NopMetricSink, QueuingMetricSink, StatsdClient};
use cadence_macros::statsd_count;
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_global_statsdclient_queuing(c: &mut Criterion) {
    let client = StatsdClient::from_sink("client.bench", QueuingMetricSink::from(NopMetricSink));
    cadence_macros::set_global_default(client);

    // NOTE: We're using counters here as representative of the performance of all types
    // of metrics which tends to be accurate except in special cases (like f64 gauges or
    // timers and histograms using Durations).

    c.bench_function("macros_statsdclient_queuing_statsd_counter", |b| {
        b.iter(|| {
            statsd_count!("some.counter", 123);
        })
    });

    c.bench_function("macros_statsdclient_queuing_statsd_counter_tags", |b| {
        b.iter(|| {
            statsd_count!("some.counter", 123, "tag" => "val", "another" => "thing");
        })
    });
}

criterion_group!(benches, benchmark_global_statsdclient_queuing,);

criterion_main!(benches);

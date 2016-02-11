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
    Counter,
    Timer,
    Gauge,
    Meter,
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

#[bench]
fn test_benchmark_new_counter_obj(b: &mut Bencher) {
    b.iter(|| {
        Counter::new("prefix", "some.counter", 5);
    });
}

#[bench]
fn test_benchmark_new_timer_obj(b: &mut Bencher) {
    b.iter(|| {
        Timer::new("prefix", "some.timer", 5);
    });

}


#[bench]
fn test_benchmark_new_gauge_obj(b: &mut Bencher) {
    b.iter(|| {
        Gauge::new("prefix", "some.gauge", 5);
    });

}


#[bench]
fn test_benchmark_new_meter_obj(b: &mut Bencher) {
    b.iter(|| {
        Meter::new("prefix", "some.meter", 5);
    });

}

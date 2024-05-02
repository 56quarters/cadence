use cadence::{SpyMetricSink, StatsdClient};
use cadence_macros::{
    statsd_count, statsd_distribution, statsd_gauge, statsd_histogram, statsd_meter, statsd_set, statsd_time,
    SingletonHolder,
};
use crossbeam_channel::Receiver;
use std::collections::HashSet;
use std::time::Duration;

static RX: SingletonHolder<Receiver<Vec<u8>>> = SingletonHolder::new();

/// Set a default client and save a reference to a channel for inspecting metrics
fn init_default_client() {
    // Save a global reference to the receiver that the spy sink will make any
    // written metrics available in.
    let (rx, sink) = SpyMetricSink::new();
    RX.set(rx);

    cadence_macros::set_global_default(StatsdClient::from_sink("my.prefix", sink));
}

/// Get the all strings written to the sink so far.
fn read_all_metrics() -> HashSet<String> {
    let rx = RX.get().unwrap();

    // We use a SpyMetricSink above (non-buffered) for the global client so each metric
    // is a separate string that we can read from the channel and look for in our tests.
    let mut out = HashSet::new();
    while let Ok(v) = rx.try_recv() {
        out.insert(String::from_utf8(v).unwrap());
    }

    out
}

#[test]
fn test_macros() {
    // NOTE: We're testing all the macros as part of a single #[test] block
    // because test functions are run in multiple threads by default and we
    // wouldn't be able to guarantee which metrics were in the rx buffer when
    // the test ran otherwise.
    init_default_client();

    fn test_counter_macros() {
        statsd_count!("some.counter", 123);
        statsd_count!("some.counter", 123, "host" => "web01.example.com", "slice" => "a");

        let metrics = read_all_metrics();
        assert!(metrics.contains("my.prefix.some.counter:123|c"));
        assert!(metrics.contains("my.prefix.some.counter:123|c|#host:web01.example.com,slice:a"));
    }

    fn test_timer_macros() {
        statsd_time!("some.timer", 334);
        statsd_time!("some.timer", 334, "type" => "api", "status" => "200");
        statsd_time!("some.timer", Duration::from_millis(334), "type" => "web");

        let metrics = read_all_metrics();
        assert!(metrics.contains("my.prefix.some.timer:334|ms"));
        assert!(metrics.contains("my.prefix.some.timer:334|ms|#type:api,status:200"));
        assert!(metrics.contains("my.prefix.some.timer:334|ms|#type:web"));
    }

    fn test_gauge_macros() {
        statsd_gauge!("some.gauge", 42);
        statsd_gauge!("some.gauge", 42, "org" => "123", "service" => "gateway");
        statsd_gauge!("some.gauge", 35.4, "org" => "456");

        let metrics = read_all_metrics();
        assert!(metrics.contains("my.prefix.some.gauge:42|g"));
        assert!(metrics.contains("my.prefix.some.gauge:42|g|#org:123,service:gateway"));
        assert!(metrics.contains("my.prefix.some.gauge:35.4|g|#org:456"));
    }

    fn test_meter_macros() {
        statsd_meter!("some.meter", 1);
        statsd_meter!("some.meter", 1, "foo" => "bar", "result" => "reject");

        let metrics = read_all_metrics();
        assert!(metrics.contains("my.prefix.some.meter:1|m"));
        assert!(metrics.contains("my.prefix.some.meter:1|m|#foo:bar,result:reject"));
    }

    fn test_histogram_macros() {
        statsd_histogram!("some.histogram", 223);
        statsd_histogram!("some.histogram", 223, "method" => "auth", "result" => "error");
        statsd_histogram!("some.histogram", Duration::from_nanos(223), "method" => "list");
        statsd_histogram!("some.histogram", 22.3, "method" => "list");

        let metrics = read_all_metrics();
        assert!(metrics.contains("my.prefix.some.histogram:223|h"));
        assert!(metrics.contains("my.prefix.some.histogram:223|h|#method:auth,result:error"));
        assert!(metrics.contains("my.prefix.some.histogram:223|h|#method:list"));
        assert!(metrics.contains("my.prefix.some.histogram:22.3|h|#method:list"));
    }

    fn test_distribution_macros() {
        statsd_distribution!("some.distribution", 22);
        statsd_distribution!("some.distribution", 22, "method" => "auth", "result" => "error");
        statsd_distribution!("some.distribution", 2.21, "method" => "list");

        let metrics = read_all_metrics();
        assert!(metrics.contains("my.prefix.some.distribution:22|d"));
        assert!(metrics.contains("my.prefix.some.distribution:22|d|#method:auth,result:error"));
        assert!(metrics.contains("my.prefix.some.distribution:2.21|d|#method:list"));
    }

    fn test_set_macros() {
        statsd_set!("some.set", 348);
        statsd_set!("some.set", 348, "service" => "user", "host" => "app01.example.com");

        let metrics = read_all_metrics();
        assert!(metrics.contains("my.prefix.some.set:348|s"));
        assert!(metrics.contains("my.prefix.some.set:348|s|#service:user,host:app01.example.com"));
    }

    test_counter_macros();
    test_timer_macros();
    test_gauge_macros();
    test_meter_macros();
    test_histogram_macros();
    test_distribution_macros();
    test_set_macros();
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::fmt;
use std::net::{ToSocketAddrs, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use ::builder::{MetricBuilder, MetricFormatter};

use ::sinks::{MetricSink, UdpMetricSink};

use ::types::{MetricResult, MetricError, ErrorKind, Counter, Timer, Gauge,
              Meter, Histogram, Metric};


/// Trait for incrementing and decrementing counters.
///
/// Counters are simple values incremented or decremented by a client. The
/// rates at which these events occur or average values will be determined
/// by the server receiving them. Examples of counter uses include number
/// of logins to a system or requests received.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait Counted {
    /// Increment the counter by `1`
    fn incr(&self, key: &str) -> MetricResult<Counter>;
    fn incr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<Counter>;

    /// Decrement the counter by `1`
    fn decr(&self, key: &str) -> MetricResult<Counter>;
    fn decr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<Counter>;

    /// Increment or decrement the counter by the given amount
    fn count(&self, key: &str, count: i64) -> MetricResult<Counter>;
    fn count_with_tags<'a>(&'a self, key: &'a str, count: i64) -> MetricBuilder<Counter>;
}


/// Trait for recording timings in milliseconds.
///
/// Timings are a positive number of milliseconds between a start and end
/// time. Examples include time taken to render a web page or time taken
/// for a database call to return.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait Timed {
    /// Record a timing in milliseconds with the given key
    fn time(&self, key: &str, time: u64) -> MetricResult<Timer>;
    fn time_with_tags<'a>(&'a self, key: &'a str, time: u64) -> MetricBuilder<Timer>;

    /// Record a timing in milliseconds with the given key
    ///
    /// The duration will be truncated to millisecond precision. If the
    /// duration cannot be represented as a `u64` an error will be returned.
    fn time_duration(&self, key: &str, duration: Duration) -> MetricResult<Timer>;
    fn time_duration_with_tags<'a>(
        &'a self,
        key: &'a str,
        duration: Duration,
    ) -> MetricBuilder<Timer>;
}


/// Trait for recording gauge values.
///
/// Gauge values are an instantaneous measurement of a value determined
/// by the client. They do not change unless changed by the client. Examples
/// include things like load average or how many connections are active.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait Gauged {
    /// Record a gauge value with the given key
    fn gauge(&self, key: &str, value: u64) -> MetricResult<Gauge>;
    fn gauge_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Gauge>;
}


/// Trait for recording meter values.
///
/// Meter values measure the rate at which events occur. These rates are
/// determined by the server, the client simply indicates when they happen.
/// Meters can be thought of as increment-only counters. Examples include
/// things like number of requests handled or number of times something is
/// flushed to disk.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait Metered {
    /// Record a single metered event with the given key
    fn mark(&self, key: &str) -> MetricResult<Meter>;
    fn mark_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<Meter>;

    /// Record a meter value with the given key
    fn meter(&self, key: &str, value: u64) -> MetricResult<Meter>;
    fn meter_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Meter>;
}


/// Trait for recording histogram values.
///
/// Histogram values are positive values that can represent anything, whose
/// statistical distribution is calculated by the server. The values can be
/// timings, amount of some resource consumed, size of HTTP responses in
/// some application, etc. Histograms can be thought of as a more general
/// form of timers. They are an extension to the Statsd protocol so you
/// should check if your server supports them before using them.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait Histogrammed {
    /// Record a single histogram value with the given key
    fn histogram(&self, key: &str, value: u64) -> MetricResult<Histogram>;
    fn histogram_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Histogram>;
}


/// Trait that encompasses all other traits for sending metrics.
///
/// If you wish to use `StatsdClient` with a generic type or place a
/// `StatsdClient` instance behind a pointer (such as a `Box`) this will allow
/// you to reference all the implemented methods for recording metrics, while
/// using a single trait. An example of this is shown below.
///
/// ```
/// use cadence::{MetricClient, StatsdClient, NopMetricSink};
///
/// let client: Box<MetricClient> = Box::new(StatsdClient::from_sink(
///     "prefix", NopMetricSink));
///
/// client.count("some.counter", 1).unwrap();
/// client.time("some.timer", 42).unwrap();
/// client.gauge("some.gauge", 8).unwrap();
/// client.meter("some.meter", 13).unwrap();
/// client.histogram("some.histogram", 4).unwrap();
/// ```
pub trait MetricClient: Counted + Timed + Gauged + Metered + Histogrammed {}


/// Client for Statsd that implements various traits to record metrics.
///
/// # Traits
///
/// The client is the main entry point for users of this library. It supports
/// several traits for recording metrics of different types.
///
/// * `Counted` for emitting counters.
/// * `Timed` for emitting timings.
/// * `Gauged` for emitting gauge values.
/// * `Metered` for emitting meter values.
/// * `Histogrammed` for emitting histogram values.
/// * `MetricClient` for a combination of all of the above.
///
/// For more information about the uses for each type of metric, see the
/// documentation for each mentioned trait.
///
/// # Sinks
///
/// The client uses some implementation of a `MetricSink` to emit the metrics.
///
/// In simple use cases when performance isn't critical, the `UdpMetricSink`
/// is an acceptable choice since it is the simplest to use and understand.
///
/// When performance is more important, users will want to use the
/// `BufferedUdpMetricSink` in combination with the `QueuingMetricSink` for
/// maximum isolation between the sending of metrics and your application as well
/// as minimum overhead when sending metrics.
///
/// # Threading
///
/// The `StatsdClient` is designed to work in a multithreaded application. All
/// parts of the client can be shared between threads (i.e. it is `Send` and
/// `Sync`). Some common ways to use the client in a multithreaded environment
/// are given below.
///
/// In each of these examples, we create a struct `MyRequestHandler` that has a
/// single method that spawns a thread to do some work and emit a metric.
///
/// ## Wrapping With An `Arc`
///
/// One option is to put all accesses to the client behind an atomic reference
/// counting pointer (`std::sync::Arc`). If you are doing this, it makes sense
/// to just refer to the client by the trait of all its methods for recording
/// metrics (`MetricClient`) as well as the `Send` and `Sync` traits since the
/// idea is to share this between threads.
///
/// ``` no_run
/// use std::net::UdpSocket;
/// use std::sync::Arc;
/// use std::thread;
/// use cadence::prelude::*;
/// use cadence::{StatsdClient, BufferedUdpMetricSink, DEFAULT_PORT};
///
/// struct MyRequestHandler {
///     metrics: Arc<MetricClient + Send + Sync>,
/// }
///
/// impl MyRequestHandler {
///     fn new() -> MyRequestHandler {
///         let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
///         let host = ("localhost", DEFAULT_PORT);
///         let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
///         MyRequestHandler {
///             metrics: Arc::new(StatsdClient::from_sink("some.prefix", sink))
///         }
///     }
///
///     fn handle_some_request(&self) -> Result<(), String> {
///         let metric_ref = self.metrics.clone();
///         let _t = thread::spawn(move || {
///             println!("Hello from the thread!");
///             metric_ref.incr("request.handler");
///         });
///
///         Ok(())
///     }
/// }
/// ```
///
/// ## Clone Per Thread
///
/// Another option for sharing the client between threads is just to clone
/// client itself. Clones of the client are relatively cheap, typically only
/// requiring a single heap allocation (of a `String`). While this cost isn't
/// nothing, it's not too bad. An example of this is given below.
///
/// ``` no_run
/// use std::net::UdpSocket;
/// use std::thread;
/// use cadence::prelude::*;
/// use cadence::{StatsdClient, BufferedUdpMetricSink, DEFAULT_PORT};
///
/// struct MyRequestHandler {
///     metrics: StatsdClient,
/// }
///
/// impl MyRequestHandler {
///     fn new() -> MyRequestHandler {
///         let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
///         let host = ("localhost", DEFAULT_PORT);
///         let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
///         MyRequestHandler {
///             metrics: StatsdClient::from_sink("some.prefix", sink)
///         }
///     }
///
///     fn handle_some_request(&self) -> Result<(), String> {
///         let metric_clone = self.metrics.clone();
///         let _t = thread::spawn(move || {
///             println!("Hello from the thread!");
///             metric_clone.incr("request.handler");
///         });
///
///         Ok(())
///     }
/// }
/// ```
///
/// As you can see, cloning the client itself looks a lot like using it with
/// an `Arc`.
#[derive(Clone)]
pub struct StatsdClient {
    prefix: String,
    sink: Arc<MetricSink + Sync + Send>,
}


impl StatsdClient {
    /// Create a new client instance that will use the given prefix for
    /// all metrics emitted to the given `MetricSink` implementation.
    ///
    /// # No-op Example
    ///
    /// ```
    /// use cadence::{StatsdClient, NopMetricSink};
    ///
    /// let prefix = "my.stats";
    /// let client = StatsdClient::from_sink(prefix, NopMetricSink);
    /// ```
    ///
    /// # UDP Socket Example
    ///
    /// ```
    /// use std::net::UdpSocket;
    /// use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
    ///
    /// let prefix = "my.stats";
    /// let host = ("127.0.0.1", DEFAULT_PORT);
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// socket.set_nonblocking(true).unwrap();
    ///
    /// let sink = UdpMetricSink::from(host, socket).unwrap();
    /// let client = StatsdClient::from_sink(prefix, sink);
    /// ```
    ///
    /// # Buffered UDP Socket Example
    ///
    /// ```
    /// use std::net::UdpSocket;
    /// use cadence::{StatsdClient, BufferedUdpMetricSink, DEFAULT_PORT};
    ///
    /// let prefix = "my.stats";
    /// let host = ("127.0.0.1", DEFAULT_PORT);
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    ///
    /// let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    /// let client = StatsdClient::from_sink(prefix, sink);
    /// ```
    pub fn from_sink<T>(prefix: &str, sink: T) -> StatsdClient
        where T: MetricSink + Sync + Send + 'static
    {
        StatsdClient {
            prefix: trim_key(prefix).to_string(),
            sink: Arc::new(sink),
        }
    }

    /// Create a new client instance that will use the given prefix to send
    /// metrics to the given host over UDP using an appropriate sink.
    ///
    /// The created UDP socket will be put into non-blocking mode.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cadence::{StatsdClient, UdpMetricSink};
    ///
    /// let prefix = "my.stats";
    /// let host = ("metrics.example.com", 8125);
    ///
    /// let client = StatsdClient::from_udp_host(prefix, host);
    /// ```
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to create a local UDP socket.
    /// * It is unable to put the UDP socket into non-blocking mode.
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed.
    pub fn from_udp_host<A>(prefix: &str, host: A) -> MetricResult<StatsdClient>
        where A: ToSocketAddrs
    {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let sink = UdpMetricSink::from(host, socket)?;
        Ok(StatsdClient::from_sink(prefix, sink))
    }

    // Convert a metric to its Statsd string representation and then send
    // it as UTF-8 bytes to the metric sink. Convert any I/O errors from the
    // sink to a MetricResult.
    pub(crate) fn send_metric<M: Metric>(&self, metric: &M) -> MetricResult<()> {
        let metric_string = metric.as_metric_str();
        self.sink.emit(metric_string)?;
        Ok(())
    }
}


impl fmt::Debug for StatsdClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StatsdClient {{ prefix: {:?}, sink: ... }}", self.prefix)
    }
}


impl Counted for StatsdClient {
    fn incr(&self, key: &str) -> MetricResult<Counter> {
        self.count(key, 1)
    }

    fn incr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<Counter> {
        self.count_with_tags(key, 1)
    }

    fn decr(&self, key: &str) -> MetricResult<Counter> {
        self.count(key, -1)
    }

    fn decr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<Counter> {
        self.count_with_tags(key, -1)
    }

    fn count(&self, key: &str, count: i64) -> MetricResult<Counter> {
        self.count_with_tags(key, count).send()
    }

    fn count_with_tags<'a>(&'a self, key: &'a str, count: i64) -> MetricBuilder<Counter> {
        let fmt = MetricFormatter::counter(&self.prefix, key, count);
        MetricBuilder::new(fmt, self)
    }
}


impl Timed for StatsdClient {
    fn time(&self, key: &str, time: u64) -> MetricResult<Timer> {
        self.time_with_tags(key, time).send()
    }

    fn time_with_tags<'a>(&'a self, key: &'a str, time: u64) -> MetricBuilder<Timer> {
        let fmt = MetricFormatter::timer(&self.prefix, key, time);
        MetricBuilder::new(fmt, self)
    }

    fn time_duration(&self, key: &str, duration: Duration) -> MetricResult<Timer> {
        self.time_duration_with_tags(key, duration).send()
    }

    fn time_duration_with_tags<'a>(
        &'a self,
        key: &'a str,
        duration: Duration,
    ) -> MetricBuilder<Timer> {
        let secs_as_ms = duration.as_secs().checked_mul(1_000);
        let nanos_as_ms = u64::from(duration.subsec_nanos()).checked_div(1_000_000);

        let result = secs_as_ms
            .and_then(|v1| nanos_as_ms.and_then(|v2| v1.checked_add(v2)))
            .ok_or_else(|| MetricError::from((ErrorKind::InvalidInput, "u64 overflow")));
        match result {
            Ok(millis) => self.time_with_tags(key, millis),
            Err(e) => MetricBuilder::from_error(e, self)
        }
    }
}


impl Gauged for StatsdClient {
    fn gauge(&self, key: &str, value: u64) -> MetricResult<Gauge> {
        self.gauge_with_tags(key, value).send()
    }

    fn gauge_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Gauge> {
        let fmt = MetricFormatter::gauge(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}


impl Metered for StatsdClient {
    fn mark(&self, key: &str) -> MetricResult<Meter> {
        self.mark_with_tags(key).send()
    }

    fn mark_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<Meter> {
        self.meter_with_tags(key, 1)
    }

    fn meter(&self, key: &str, value: u64) -> MetricResult<Meter> {
        self.meter_with_tags(key, value).send()
    }

    fn meter_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Meter> {
        let fmt = MetricFormatter::meter(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}


impl Histogrammed for StatsdClient {
    fn histogram(&self, key: &str, value: u64) -> MetricResult<Histogram> {
        self.histogram_with_tags(key, value).send()
    }

    fn histogram_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<Histogram> {
        let fmt = MetricFormatter::histogram(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}


impl MetricClient for StatsdClient {}


fn trim_key(val: &str) -> &str {
    if val.ends_with('.') {
        val.trim_right_matches('.')
    } else {
        val
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::u64;
    use super::{trim_key, Counted, Timed, Gauged, Metered, Histogrammed,
                MetricClient, StatsdClient};
    use ::sinks::NopMetricSink;
    use ::types::{ErrorKind, Timer};

    #[test]
    fn test_trim_key_with_trailing_dot() {
        assert_eq!("some.prefix", trim_key("some.prefix."));
    }

    #[test]
    fn test_trim_key_no_trailing_dot() {
        assert_eq!("some.prefix", trim_key("some.prefix"));
    }

    // The following tests really just ensure that we've actually
    // implemented all the traits we're supposed to correctly. If
    // we hadn't, this wouldn't compile.

    #[test]
    fn test_statsd_client_as_counted() {
        let client: Box<Counted> = Box::new(StatsdClient::from_sink(
            "prefix", NopMetricSink));

        client.count("some.counter", 5).unwrap();
    }

    #[test]
    fn test_statsd_client_as_timed() {
        let client: Box<Timed> = Box::new(StatsdClient::from_sink(
            "prefix", NopMetricSink));

        client.time("some.timer", 20).unwrap();
    }

    #[test]
    fn test_statsd_client_as_gauged() {
        let client: Box<Gauged> = Box::new(StatsdClient::from_sink(
            "prefix", NopMetricSink));

        client.gauge("some.gauge", 32).unwrap();
    }

    #[test]
    fn test_statsd_client_as_metered() {
        let client: Box<Metered> = Box::new(StatsdClient::from_sink(
            "prefix", NopMetricSink));

        client.meter("some.meter", 9).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed() {
        let client: Box<Histogrammed> = Box::new(StatsdClient::from_sink(
            "prefix", NopMetricSink));

        client.histogram("some.histogram", 4).unwrap();
    }

    #[test]
    fn test_statsd_client_as_metric_client() {
        let client: Box<MetricClient> = Box::new(StatsdClient::from_sink(
            "prefix", NopMetricSink));

        client.count("some.counter", 3).unwrap();
        client.time("some.timer", 198).unwrap();
        client.gauge("some.gauge", 4).unwrap();
        client.meter("some.meter", 29).unwrap();
        client.histogram("some.histogram", 32).unwrap();
    }

    #[test]
    fn test_statsd_client_time_duration_no_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.time_duration("key", Duration::from_millis(157));
        let expected = Timer::new("prefix", "key", 157);

        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_statsd_client_time_duration_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.time_duration("key", Duration::from_secs(u64::MAX));
        let err = res.unwrap_err();

        assert_eq!(ErrorKind::InvalidInput, err.kind())
    }

    #[test]
    fn test_statsd_client_with_tags() {
        let client: Box<MetricClient> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.incr_with_tags("some.counter").send().unwrap();
        client
            .count_with_tags("some.counter", 3)
            .with_tag("foo", "bar")
            .send()
            .unwrap();
        client
            .time_with_tags("some.timer", 22)
            .with_tag("host", "app01.example.com")
            .with_tag("bucket", "A")
            .send()
            .unwrap();
        client
            .gauge_with_tags("some.gauge", 4)
            .with_tag("bucket", "A")
            .with_tag_value("file-server")
            .send()
            .unwrap();
    }

    #[test]
    fn test_statsd_client_time_duration_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        client
            .time_duration_with_tags("key", Duration::from_millis(157))
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .send()
            .unwrap();
    }

    #[test]
    fn test_statsd_client_time_duration_with_tags_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .time_duration_with_tags("key", Duration::from_secs(u64::MAX))
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .send();
        assert!(res.is_err());
        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }
}

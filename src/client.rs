// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2020 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::net::{ToSocketAddrs, UdpSocket};
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use std::time::Duration;
use std::u64;

use crate::builder::{MetricBuilder, MetricFormatter};
use crate::sinks::{MetricSink, UdpMetricSink};
use crate::types::{
    Counter, ErrorKind, Gauge, Histogram, Meter, Metric, MetricError, MetricResult, Set, Timer,
};

/// Trait for incrementing and decrementing counters.
///
/// Counters are simple values incremented or decremented by a client. The
/// rates at which these events occur or average values will be determined
/// by the server receiving them. Examples of counter uses include number
/// of logins to a system or requests received.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Counted {
    /// Increment the counter by `1`
    fn incr(&self, key: &str) -> MetricResult<Counter> {
        self.count(key, 1)
    }

    /// Increment the counter by `1` and return a `MetricBuilder` that can
    /// be used to add tags to the metric.
    fn incr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<'_, '_, Counter> {
        self.count_with_tags(key, 1)
    }

    /// Decrement the counter by `1`
    fn decr(&self, key: &str) -> MetricResult<Counter> {
        self.count(key, -1)
    }

    /// Decrement the counter by `1` and return a `MetricBuilder that can
    /// be used to add tags to the metric.
    fn decr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<'_, '_, Counter> {
        self.count_with_tags(key, -1)
    }

    /// Increment or decrement the counter by the given amount
    fn count(&self, key: &str, count: i64) -> MetricResult<Counter> {
        self.count_with_tags(key, count).try_send()
    }

    /// Increment or decrement the counter by the given amount and return
    /// a `MetricBuilder` that can be used to add tags to the metric.
    fn count_with_tags<'a>(&'a self, key: &'a str, count: i64) -> MetricBuilder<'_, '_, Counter>;
}

/// Trait for recording timings in milliseconds.
///
/// Timings are a positive number of milliseconds between a start and end
/// time. Examples include time taken to render a web page or time taken
/// for a database call to return.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Timed {
    /// Record a timing in milliseconds with the given key
    fn time(&self, key: &str, time: u64) -> MetricResult<Timer> {
        self.time_with_tags(key, time).try_send()
    }

    /// Record a timing in milliseconds with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn time_with_tags<'a>(&'a self, key: &'a str, time: u64) -> MetricBuilder<'_, '_, Timer>;

    /// Record a timing in milliseconds with the given key
    ///
    /// The duration will be truncated to millisecond precision. If the
    /// duration cannot be represented as a `u64` an error will be returned.
    fn time_duration(&self, key: &str, duration: Duration) -> MetricResult<Timer> {
        self.time_duration_with_tags(key, duration).try_send()
    }

    /// Record a timing in milliseconds with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    ///
    /// The duration will be truncated to millisecond precision. If the
    /// duration cannot be represented as a `u64` an error will be deferred
    /// and returned when `MetricBuilder::try_send()` is called.
    fn time_duration_with_tags<'a>(
        &'a self,
        key: &'a str,
        duration: Duration,
    ) -> MetricBuilder<'_, '_, Timer>;
}

/// Trait for recording gauge values.
///
/// Gauge values are an instantaneous measurement of a value determined
/// by the client. They do not change unless changed by the client. Examples
/// include things like load average or how many connections are active.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Gauged {
    /// Record a gauge value with the given key
    fn gauge(&self, key: &str, value: u64) -> MetricResult<Gauge> {
        self.gauge_with_tags(key, value).try_send()
    }

    /// Record a gauge value with the given key and return a `MetricBuilder`
    /// that can be used to add tags to the metric.
    fn gauge_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<'_, '_, Gauge>;
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
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Metered {
    /// Record a single metered event with the given key
    fn mark(&self, key: &str) -> MetricResult<Meter> {
        self.meter(key, 1)
    }

    /// Record a single metered event with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn mark_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<'_, '_, Meter> {
        self.meter_with_tags(key, 1)
    }

    /// Record a meter value with the given key
    fn meter(&self, key: &str, value: u64) -> MetricResult<Meter> {
        self.meter_with_tags(key, value).try_send()
    }

    /// Record a meter value with the given key and return a `MetricBuilder`
    /// that can be used to add tags to the metric.
    fn meter_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<'_, '_, Meter>;
}

/// Trait for recording histogram values.
///
/// Histogram values are positive values that can represent anything, whose
/// statistical distribution is calculated by the server. The values can be
/// timings, amount of some resource consumed, size of HTTP responses in
/// some application, etc. Histograms can be thought of as a more general
/// form of timers.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags and histograms are a
/// [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) extension to
/// Statsd and may not be supported by your server.
pub trait Histogrammed {
    /// Record a single histogram value with the given key
    fn histogram(&self, key: &str, value: u64) -> MetricResult<Histogram> {
        self.histogram_with_tags(key, value).try_send()
    }

    /// Record a single histogram value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn histogram_with_tags<'a>(
        &'a self,
        key: &'a str,
        value: u64,
    ) -> MetricBuilder<'_, '_, Histogram>;

    /// Record a single histogram value with the given key.
    ///
    /// The duration will be converted to nanoseconds. If the duration
    /// cannot be represented as a `u64` an error will be returned. Note
    /// that histograms are an extension to Statsd, you'll need to check
    /// if they are supported by your server and considered times.
    fn histogram_duration(&self, key: &str, duration: Duration) -> MetricResult<Histogram> {
        self.histogram_duration_with_tags(key, duration).try_send()
    }

    /// Record a single histogram value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    ///
    /// The duration will be converted to nanoseconds. If the duration cannot
    /// be represented as a `u64` an error will be deferred and returned when
    /// `MetricBuilder::try_send()` is called. Note that histograms are an
    /// extension to Statsd, you'll need to check if they are supported by
    /// your server and considered times.
    fn histogram_duration_with_tags<'a>(
        &'a self,
        key: &'a str,
        duration: Duration,
    ) -> MetricBuilder<'_, '_, Histogram>;
}

/// Trait for recording set values.
///
/// Sets count the number of unique elements in a group. You can use them to,
/// for example, count the unique visitors to your site.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait Setted {
    /// Record a single set value with the given key
    fn set(&self, key: &str, value: i64) -> MetricResult<Set> {
        self.set_with_tags(key, value).try_send()
    }

    /// Record a single set value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn set_with_tags<'a>(&'a self, key: &'a str, value: i64) -> MetricBuilder<'_, '_, Set>;
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
/// client.set("some.set", 5).unwrap();
/// ```
pub trait MetricClient: Counted + Timed + Gauged + Metered + Histogrammed + Setted {}

/// Typically internal methods for sending metrics and handling errors.
///
/// This trait exposes methods of the client that would normally be internal
/// but may be useful for consumers of the library to extend it in unforseen
/// ways.
///
/// This trait is not exposed in the `prelude` module since it isn't required
/// to use the client for sending metrics. It is only exposed in the `ext`
/// module which is used to encompass extension points for the library.
///
/// # Example
///
/// ```
/// use cadence::{Metric, MetricResult, StatsdClient, NopMetricSink};
/// use cadence::ext::MetricBackend;
///
/// struct CustomMetric {
///     repr: String,
/// }
///
/// impl Metric for CustomMetric {
///     fn as_metric_str(&self) -> &str {
///         &self.repr
///     }
/// }
///
/// impl From<String> for CustomMetric {
///     fn from(v: String) -> Self {
///         CustomMetric { repr: v }
///     }
/// }
///
/// struct MyCustomClient {
///     prefix: String,
///     wrapped: StatsdClient,
/// }
///
/// impl MyCustomClient {
///     fn new(prefix: &str, client: StatsdClient) -> Self {
///         MyCustomClient {
///             prefix: prefix.to_string(),
///             wrapped: client,
///         }
///     }
///
///     fn send_event(&self, key: &str, val: i64) -> MetricResult<CustomMetric> {
///         let metric = CustomMetric::from(format!("{}.{}:{}|e", self.prefix, key, val));
///         self.wrapped.send_metric(&metric)?;
///         Ok(metric)
///     }
///
///     fn send_event_quietly(&self, key: &str, val: i64) {
///         if let Err(e) = self.send_event(key, val) {
///             self.wrapped.consume_error(e);
///         }
///     }
/// }
///
/// let prefix = "some.prefix";
/// let inner = StatsdClient::from_sink(&prefix, NopMetricSink);
/// let custom = MyCustomClient::new(&prefix, inner);
///
/// custom.send_event("some.event", 123).unwrap();
/// custom.send_event_quietly("some.event", 456);
/// ```
pub trait MetricBackend {
    /// Send a full formed `Metric` implementation via the underlying `MetricSink`
    ///
    /// Obtain a `&str` representation of a metric, encode it as UTF-8 bytes, and
    /// send it to the underlying `MetricSink`, verbatim. Note that the metric is
    /// expected to be full formed already, including any prefix or tags.
    ///
    /// Note that if you simply want to emit standard metrics, you don't need to
    /// use this method. This is only useful if you are extending Cadence with a
    /// custom metric type or something similar.
    fn send_metric<M>(&self, metric: &M) -> MetricResult<()>
    where
        M: Metric;

    /// Consume a possible error from attempting to send a metric.
    ///
    /// When callers have elected to quietly send metrics via the `MetricBuilder::send()`
    /// method, this method will be invoked if an error is encountered. By default the
    /// handler is a no-op, meaning that errors are discarded.
    ///
    /// Note that if you simply want to emit standard metrics, you don't need to
    /// use this method. This is only useful if you are extending Cadence with a
    /// custom metric type or something similar.
    fn consume_error(&self, err: MetricError);
}

/// Builder for creating and customizing `StatsdClient` instances.
///
/// Instances of the builder should be created by calling the `::builder()`
/// method on the `StatsClient` struct.
///
/// # Example
///
/// ```
/// use cadence::prelude::*;
/// use cadence::{MetricError, StatsdClient, NopMetricSink};
///
/// fn my_error_handler(err: MetricError) {
///     println!("Metric error! {}", err);
/// }
///
/// let client = StatsdClient::builder("prefix", NopMetricSink)
///     .with_error_handler(my_error_handler)
///     .build();
///
/// client.count("something", 123);
/// client.count_with_tags("some.counter", 42)
///     .with_tag("region", "us-east-2")
///     .send();
/// ```
pub struct StatsdClientBuilder {
    prefix: String,
    sink: Box<dyn MetricSink + Sync + Send + RefUnwindSafe>,
    errors: Box<dyn Fn(MetricError) -> () + Sync + Send + RefUnwindSafe>,
}

impl StatsdClientBuilder {
    // Set the required fields and defaults for optional fields
    fn new<T>(prefix: &str, sink: T) -> Self
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        StatsdClientBuilder {
            // required
            prefix: Self::get_formatted_prefix(prefix),
            sink: Box::new(sink),

            // optional with defaults
            errors: Box::new(nop_error_handler),
        }
    }

    /// Set an error handler to use for metrics sent via `MetricBuilder::send()`
    ///
    /// The error handler is only invoked when metrics are not able to be sent
    /// correctly. Either due to invalid input, I/O errors encountered when trying
    /// to send them via a `MetricSink`, or some other reason.
    ///
    /// The error handler should consume the error without panicking. The error
    /// may be logged, printed to stderr, discarded, etc. - this is up to the
    /// implementation.
    pub fn with_error_handler<F>(mut self, errors: F) -> Self
    where
        F: Fn(MetricError) -> () + Sync + Send + RefUnwindSafe + 'static,
    {
        self.errors = Box::new(errors);
        self
    }

    /// Construct a new `StatsdClient` instance based on current settings.
    pub fn build(self) -> StatsdClient {
        StatsdClient::from_builder(self)
    }

    fn get_formatted_prefix(prefix: &str) -> String {
        if prefix.is_empty() {
            String::new()
        } else {
            format!("{}.", prefix.trim_end_matches('.'))
        }
    }
}

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
/// * `Setted` for emitting set values.
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
/// use std::panic::RefUnwindSafe;
/// use std::net::UdpSocket;
/// use std::sync::Arc;
/// use std::thread;
/// use cadence::prelude::*;
/// use cadence::{StatsdClient, BufferedUdpMetricSink, DEFAULT_PORT};
///
/// struct MyRequestHandler {
///     metrics: Arc<MetricClient + Send + Sync + RefUnwindSafe>,
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
    sink: Arc<dyn MetricSink + Sync + Send + RefUnwindSafe>,
    errors: Arc<dyn Fn(MetricError) -> () + Sync + Send + RefUnwindSafe>,
}

impl StatsdClient {
    /// Create a new client instance that will use the given prefix for
    /// all metrics emitted to the given `MetricSink` implementation.
    ///
    /// Note that this client will discard errors encountered when
    /// sending metrics via the `MetricBuilder::send()` method.
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
    pub fn from_sink<T>(prefix: &str, sink: T) -> Self
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        Self::builder(prefix, sink).build()
    }

    /// Create a new client instance that will use the given prefix to send
    /// metrics to the given host over UDP using an appropriate sink.
    ///
    /// The created UDP socket will be put into non-blocking mode.
    ///
    /// Note that this client will discard errors encountered when
    /// sending metrics via the `MetricBuilder::send()` method.
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
    #[deprecated(since = "0.19.0", note = "Superseded by ::from_sink() and ::builder()")]
    pub fn from_udp_host<A>(prefix: &str, host: A) -> MetricResult<Self>
    where
        A: ToSocketAddrs,
    {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let sink = UdpMetricSink::from(host, socket)?;
        Ok(StatsdClient::builder(prefix, sink).build())
    }

    /// Create a new builder with the provided prefix and metric sink.
    ///
    /// A prefix and a metric sink are required to create a new client
    /// instance. All other optional customizations can be set by calling
    /// methods on the returned builder. Any customizations that aren't
    /// set by the caller will use defaults.
    ///
    /// Note, though a metric prefix is required, you may pass an empty
    /// string as a prefix. In this case, the metrics emitted will use only
    /// the bare keys supplied when you call the various methods to emit
    /// metrics.
    ///
    /// General defaults:
    ///
    /// * A no-op error handler will be used by default. Note that this
    ///   only affects errors encountered when using the `MetricBuilder::send()`
    ///   method (as opposed to `.try_send()` or any other method for sending
    ///   metrics).
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, MetricError, NopMetricSink};
    ///
    /// fn my_handler(err: MetricError) {
    ///     println!("Metric error: {}", err);
    /// }
    ///
    /// let client = StatsdClient::builder("some.prefix", NopMetricSink)
    ///     .with_error_handler(my_handler)
    ///     .build();
    ///
    /// client.gauge_with_tags("some.key", 7)
    ///    .with_tag("region", "us-west-1")
    ///    .send();
    /// ```
    pub fn builder<T>(prefix: &str, sink: T) -> StatsdClientBuilder
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        StatsdClientBuilder::new(prefix, sink)
    }

    // Create a new StatsdClient by consuming the builder
    fn from_builder(builder: StatsdClientBuilder) -> Self {
        StatsdClient {
            prefix: builder.prefix,
            sink: Arc::from(builder.sink),
            errors: Arc::from(builder.errors),
        }
    }
}

impl MetricBackend for StatsdClient {
    fn send_metric<M>(&self, metric: &M) -> MetricResult<()>
    where
        M: Metric,
    {
        let metric_string = metric.as_metric_str();
        self.sink.emit(metric_string)?;
        Ok(())
    }

    fn consume_error(&self, err: MetricError) {
        (self.errors)(err);
    }
}

impl fmt::Debug for StatsdClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StatsdClient {{ prefix: {:?}, sink: ..., errors: ... }}",
            self.prefix
        )
    }
}

impl Counted for StatsdClient {
    fn count_with_tags<'a>(&'a self, key: &'a str, count: i64) -> MetricBuilder<'_, '_, Counter> {
        let fmt = MetricFormatter::counter(&self.prefix, key, count);
        MetricBuilder::new(fmt, self)
    }
}

impl Timed for StatsdClient {
    fn time_with_tags<'a>(&'a self, key: &'a str, time: u64) -> MetricBuilder<'_, '_, Timer> {
        let fmt = MetricFormatter::timer(&self.prefix, key, time);
        MetricBuilder::new(fmt, self)
    }

    fn time_duration_with_tags<'a>(
        &'a self,
        key: &'a str,
        duration: Duration,
    ) -> MetricBuilder<'_, '_, Timer> {
        let as_millis = duration.as_millis();
        if as_millis > u64::MAX as u128 {
            MetricBuilder::from_error(MetricError::from((ErrorKind::InvalidInput, "u64 overflow")), self)
        } else {
            self.time_with_tags(key, as_millis as u64)
        }
    }
}

impl Gauged for StatsdClient {
    fn gauge_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<'_, '_, Gauge> {
        let fmt = MetricFormatter::gauge(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}

impl Metered for StatsdClient {
    fn meter_with_tags<'a>(&'a self, key: &'a str, value: u64) -> MetricBuilder<'_, '_, Meter> {
        let fmt = MetricFormatter::meter(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}

impl Histogrammed for StatsdClient {
    fn histogram_with_tags<'a>(
        &'a self,
        key: &'a str,
        value: u64,
    ) -> MetricBuilder<'_, '_, Histogram> {
        let fmt = MetricFormatter::histogram(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }

    fn histogram_duration_with_tags<'a>(
        &'a self,
        key: &'a str,
        duration: Duration,
    ) -> MetricBuilder<'_, '_, Histogram> {
        let as_nanos = duration.as_nanos();
        if as_nanos > u64::MAX as u128 {
            MetricBuilder::from_error(MetricError::from((ErrorKind::InvalidInput, "u64 overflow")), self)
        } else {
            self.histogram_with_tags(key, as_nanos as u64)
        }
    }
}

impl Setted for StatsdClient {
    fn set_with_tags<'a>(&'a self, key: &'a str, value: i64) -> MetricBuilder<'_, '_, Set> {
        let fmt = MetricFormatter::set(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}

impl MetricClient for StatsdClient {}

#[allow(clippy::needless_pass_by_value)]
fn nop_error_handler(_err: MetricError) {
    // nothing
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io;
    use std::panic::RefUnwindSafe;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::u64;

    use super::{
        Counted, Gauged, Histogrammed, Metered, MetricClient, Setted, StatsdClient, Timed,
    };

    use crate::sinks::{MetricSink, NopMetricSink, QueuingMetricSink};
    use crate::types::{ErrorKind, Metric, MetricError};

    #[test]
    fn test_statsd_client_empty_prefix() {
        let client = StatsdClient::from_sink("", NopMetricSink);
        let res = client.count("some.method", 1);

        assert_eq!("some.method:1|c", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_count_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .count_with_tags("some.counter", 3)
            .with_tag("foo", "bar")
            .try_send();

        assert_eq!(
            "prefix.some.counter:3|c|#foo:bar",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_gauge_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .gauge_with_tags("some.gauge", 4)
            .with_tag("bucket", "A")
            .with_tag_value("file-server")
            .try_send();

        assert_eq!(
            "prefix.some.gauge:4|g|#bucket:A,file-server",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_meter_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .meter_with_tags("some.meter", 64)
            .with_tag("segment", "142")
            .with_tag_value("beta")
            .try_send();

        assert_eq!(
            "prefix.some.meter:64|m|#segment:142,beta",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_histogram_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .histogram_with_tags("some.histo", 27)
            .with_tag("host", "www03.example.com")
            .with_tag_value("rc1")
            .try_send();

        assert_eq!(
            "prefix.some.histo:27|h|#host:www03.example.com,rc1",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_historgram_duration() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.histogram_duration("key", Duration::from_nanos(210));

        assert_eq!("prefix.key:210|h", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_histogram_duration_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.histogram_duration("key", Duration::from_secs(u64::MAX));

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_histogram_duration_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .histogram_duration_with_tags("key", Duration::from_nanos(4096))
            .with_tag("foo", "bar")
            .with_tag_value("beta")
            .try_send();

        assert_eq!(
            "prefix.key:4096|h|#foo:bar,beta",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_histogram_duration_with_tags_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .histogram_duration_with_tags("key", Duration::from_millis(u64::MAX))
            .with_tag("foo", "bar")
            .with_tag_value("beta")
            .try_send();

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_time_duration() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.time_duration("key", Duration::from_millis(157));

        assert_eq!("prefix.key:157|ms", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_time_duration_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.time_duration("key", Duration::from_secs(u64::MAX));

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind())
    }

    #[test]
    fn test_statsd_client_time_duration_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .time_duration_with_tags("key", Duration::from_millis(157))
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .try_send();

        assert_eq!(
            "prefix.key:157|ms|#foo:bar,quux",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_time_duration_with_tags_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .time_duration_with_tags("key", Duration::from_secs(u64::MAX))
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .try_send();

        assert!(res.is_err());
        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_with_tags_send_success() {
        struct StoringSink {
            metrics: Arc<Mutex<RefCell<Vec<String>>>>,
        }

        impl MetricSink for StoringSink {
            fn emit(&self, metric: &str) -> io::Result<usize> {
                let mutex = self.metrics.as_ref();
                let cell = mutex.lock().unwrap();
                cell.borrow_mut().push(metric.to_owned());
                Ok(0)
            }
        }

        fn panic_handler(err: MetricError) {
            panic!("Metric send error: {}", err);
        }

        let metrics = Arc::new(Mutex::new(RefCell::new(Vec::new())));
        let metrics_ref = Arc::clone(&metrics);
        let sink = StoringSink {
            metrics: metrics_ref,
        };
        let client = StatsdClient::builder("prefix", sink)
            .with_error_handler(panic_handler)
            .build();

        client
            .incr_with_tags("some.key")
            .with_tag("test", "a")
            .send();

        let mutex = metrics.as_ref();
        let cell = mutex.lock().unwrap();

        assert_eq!(1, cell.borrow().len());
    }

    #[test]
    fn test_statsd_client_with_tags_send_error() {
        struct ErrorSink;

        impl MetricSink for ErrorSink {
            fn emit(&self, _metric: &str) -> io::Result<usize> {
                Err(io::Error::from(io::ErrorKind::Other))
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        let count_ref = Arc::clone(&count);

        let handler = move |_err: MetricError| {
            count_ref.fetch_add(1, Ordering::Release);
        };

        let client = StatsdClient::builder("prefix", ErrorSink)
            .with_error_handler(handler)
            .build();

        client
            .incr_with_tags("some.key")
            .with_tag("tier", "web")
            .send();

        assert_eq!(1, count.load(Ordering::Acquire));
    }

    #[test]
    fn test_statsd_client_set_no_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.set("some.set", 3);

        assert_eq!("myapp.some.set:3|s", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_set_with_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .set_with_tags("some.set", 3)
            .with_tag("foo", "bar")
            .try_send();

        assert_eq!("myapp.some.set:3|s|#foo:bar", res.unwrap().as_metric_str());
    }

    // The following tests really just ensure that we've actually
    // implemented all the traits we're supposed to correctly. If
    // we hadn't, this wouldn't compile.

    #[test]
    fn test_statsd_client_as_counted() {
        let client: Box<dyn Counted> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.count("some.counter", 5).unwrap();
    }

    #[test]
    fn test_statsd_client_as_timed() {
        let client: Box<dyn Timed> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.time("some.timer", 20).unwrap();
    }

    #[test]
    fn test_statsd_client_as_gauged() {
        let client: Box<dyn Gauged> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.gauge("some.gauge", 32).unwrap();
    }

    #[test]
    fn test_statsd_client_as_metered() {
        let client: Box<dyn Metered> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.meter("some.meter", 9).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed() {
        let client: Box<dyn Histogrammed> =
            Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.histogram("some.histogram", 4).unwrap();
    }

    #[test]
    fn test_statsd_client_as_setted() {
        let client: Box<dyn Setted> = Box::new(StatsdClient::from_sink("myapp", NopMetricSink));

        client.set("some.set", 5).unwrap();
    }

    #[test]
    fn test_statsd_client_as_metric_client() {
        let client: Box<dyn MetricClient> =
            Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.count("some.counter", 3).unwrap();
        client.time("some.timer", 198).unwrap();
        client.gauge("some.gauge", 4).unwrap();
        client.meter("some.meter", 29).unwrap();
        client.histogram("some.histogram", 32).unwrap();
        client.set("some.set", 5).unwrap();
    }

    #[test]
    fn test_statsd_client_as_thread_and_panic_safe() {
        let client: Box<dyn MetricClient + Send + Sync + RefUnwindSafe> = Box::new(
            StatsdClient::from_sink("prefix", QueuingMetricSink::from(NopMetricSink)),
        );

        client.count("some.counter", 3).unwrap();
        client.time("some.timer", 198).unwrap();
        client.gauge("some.gauge", 4).unwrap();
        client.meter("some.meter", 29).unwrap();
        client.histogram("some.histogram", 32).unwrap();
        client.set("some.set", 5).unwrap();
    }
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::builder::{MetricBuilder, MetricFormatter, MetricValue};
use crate::sealed::Sealed;
use crate::sinks::MetricSink;
use crate::types::{
    Counter, Distribution, ErrorKind, Gauge, Histogram, Meter, Metric, MetricError, MetricResult, Set, Timer,
};
use std::fmt;
use std::panic::RefUnwindSafe;
use std::time::Duration;
use std::u64;

/// Conversion trait for valid values for counters
///
/// This trait must be implemented for any types that are used as counter
/// values (currently only `i64`). This trait is internal to how values are
/// formatted as part of metrics but is exposed publicly for documentation
/// purposes.
///
/// Typical use of Cadence shouldn't require interacting with this trait.
pub trait ToCounterValue {
    fn try_to_value(self) -> MetricResult<MetricValue>;
}

impl ToCounterValue for i64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Signed(self))
    }
}

/// Conversion trait for valid values for timers
///
/// This trait must be implemented for any types that are used as timer
/// values (currently `u64`, `Duration`, and `Vec`s of those types).
/// This trait is internal to how values are formatted as part of metrics
/// but is exposed publicly for documentation purposes.
///
/// Typical use of Cadence shouldn't require interacting with this trait.
pub trait ToTimerValue {
    fn try_to_value(self) -> MetricResult<MetricValue>;
}

impl ToTimerValue for u64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Unsigned(self))
    }
}

impl ToTimerValue for Vec<u64> {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::PackedUnsigned(self))
    }
}

impl ToTimerValue for Duration {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        let as_millis = self.as_millis();
        if as_millis > u64::MAX as u128 {
            Err(MetricError::from((ErrorKind::InvalidInput, "u64 overflow")))
        } else {
            Ok(MetricValue::Unsigned(as_millis as u64))
        }
    }
}

impl ToTimerValue for Vec<Duration> {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        if self.iter().any(|x| x.as_millis() > u64::MAX as u128) {
            Err(MetricError::from((ErrorKind::InvalidInput, "u64 overflow")))
        } else {
            Ok(MetricValue::PackedUnsigned(
                self.iter().map(|x| x.as_millis() as u64).collect(),
            ))
        }
    }
}

/// Conversion trait for valid values for gauges
///
/// This trait must be implemented for any types that are used as gauge
/// values (currently `u64` and `f64`). This trait is internal to how values
/// are formatted as part of metrics but is exposed publicly for documentation
/// purposes.
///
/// Typical use of Cadence shouldn't require interacting with this trait.
pub trait ToGaugeValue {
    fn try_to_value(self) -> MetricResult<MetricValue>;
}

impl ToGaugeValue for u64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Unsigned(self))
    }
}
impl ToGaugeValue for f64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Float(self))
    }
}

/// Conversion trait for valid values for meters
///
/// This trait must be implemented for any types that are used as meter
/// values (currently only `u64`). This trait is internal to how values are
/// formatted as part of metrics but is exposed publicly for documentation
/// purposes.
///
/// Typical use of Cadence shouldn't require interacting with this trait.
pub trait ToMeterValue {
    fn try_to_value(self) -> MetricResult<MetricValue>;
}

impl ToMeterValue for u64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Unsigned(self))
    }
}

/// Conversion trait for valid values for histograms
///
/// This trait must be implemented for any types that are used as histogram
/// values (currently `u64`, `f64`, `Duration`, and `Vec`s of those types).
/// This trait is internal to how values are formatted as part of metrics
/// but is exposed publicly for documentation purposes.
///
/// Typical use of Cadence shouldn't require interacting with this trait.
pub trait ToHistogramValue {
    fn try_to_value(self) -> MetricResult<MetricValue>;
}

impl ToHistogramValue for u64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Unsigned(self))
    }
}

impl ToHistogramValue for f64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Float(self))
    }
}

impl ToHistogramValue for Duration {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        let as_nanos = self.as_nanos();
        if as_nanos > u64::MAX as u128 {
            Err(MetricError::from((ErrorKind::InvalidInput, "u64 overflow")))
        } else {
            Ok(MetricValue::Unsigned(as_nanos as u64))
        }
    }
}

impl ToHistogramValue for Vec<u64> {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::PackedUnsigned(self))
    }
}

impl ToHistogramValue for Vec<f64> {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::PackedFloat(self))
    }
}

impl ToHistogramValue for Vec<Duration> {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        if self.iter().any(|x| x.as_nanos() > u64::MAX as u128) {
            Err(MetricError::from((ErrorKind::InvalidInput, "u64 overflow")))
        } else {
            Ok(MetricValue::PackedUnsigned(
                self.iter().map(|x| x.as_nanos() as u64).collect(),
            ))
        }
    }
}

/// Conversion trait for valid values for distributions
///
/// This trait must be implemented for any types that are used as distribution
/// values (currently `u64`, `f64`, and `Vec`s of those types). This trait is
/// internal to how values are formatted as part of metrics but is exposed
/// publicly for documentation purposes.
///
/// Typical use of Cadence shouldn't require interacting with this trait.
pub trait ToDistributionValue {
    fn try_to_value(self) -> MetricResult<MetricValue>;
}

impl ToDistributionValue for u64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Unsigned(self))
    }
}

impl ToDistributionValue for f64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Float(self))
    }
}

impl ToDistributionValue for Vec<u64> {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::PackedUnsigned(self))
    }
}

impl ToDistributionValue for Vec<f64> {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::PackedFloat(self))
    }
}

/// Conversion trait for valid values for sets
///
/// This trait must be implemented for any types that are used as counter
/// values (currently only `i64`). This trait is internal to how values are
/// formatted as part of metrics but is exposed publicly for documentation
/// purposes.
///
/// Typical use of Cadence shouldn't require interacting with this trait.
pub trait ToSetValue {
    fn try_to_value(self) -> MetricResult<MetricValue>;
}

impl ToSetValue for i64 {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        Ok(MetricValue::Signed(self))
    }
}

/// Trait for incrementing and decrementing counters.
///
/// Counters are simple values incremented or decremented by a client. The
/// rates at which these events occur or average values will be determined
/// by the server receiving them. Examples of counter uses include number
/// of logins to a system or requests received.
///
/// The following types are valid for counters:
/// * `i64`
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Counted<T>
where
    T: ToCounterValue,
{
    /// Increment or decrement the counter by the given amount
    fn count(&self, key: &str, count: T) -> MetricResult<Counter> {
        self.count_with_tags(key, count).try_send()
    }

    /// Increment or decrement the counter by the given amount and return
    /// a `MetricBuilder` that can be used to add tags to the metric.
    fn count_with_tags<'a>(&'a self, key: &'a str, count: T) -> MetricBuilder<'_, '_, Counter>;
}

/// Trait for convenience methods for counters
///
/// This trait specifically implements increment and decrement convenience
/// methods for counters with `i64` types.
pub trait CountedExt: Counted<i64> {
    /// Increment the counter by 1
    fn incr(&self, key: &str) -> MetricResult<Counter> {
        self.incr_with_tags(key).try_send()
    }

    /// Increment the counter by 1 and return a `MetricBuilder` that can
    /// be used to add tags to the metric.
    fn incr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<'_, '_, Counter> {
        self.count_with_tags(key, 1)
    }

    /// Decrement the counter by 1
    fn decr(&self, key: &str) -> MetricResult<Counter> {
        self.decr_with_tags(key).try_send()
    }

    /// Decrement the counter by 1 and return a `MetricBuilder` that can
    /// be used to add tags to the metric.
    fn decr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<'_, '_, Counter> {
        self.count_with_tags(key, -1)
    }
}

/// Trait for recording timings in milliseconds.
///
/// Timings are a positive number of milliseconds between a start and end
/// time. Examples include time taken to render a web page or time taken
/// for a database call to return. `Duration` values are converted to
/// milliseconds before being recorded.
///
/// The following types are valid for timers:
/// * `u64`
/// * `Duration`
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Timed<T>
where
    T: ToTimerValue,
{
    /// Record a timing in milliseconds with the given key
    fn time(&self, key: &str, time: T) -> MetricResult<Timer> {
        self.time_with_tags(key, time).try_send()
    }

    /// Record a timing in milliseconds with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn time_with_tags<'a>(&'a self, key: &'a str, time: T) -> MetricBuilder<'_, '_, Timer>;
}

/// Trait for recording gauge values.
///
/// Gauge values are an instantaneous measurement of a value determined
/// by the client. They do not change unless changed by the client. Examples
/// include things like load average or how many connections are active.
///
/// The following types are valid for gauges:
/// * `u64`
/// * `f64`
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Gauged<T>
where
    T: ToGaugeValue,
{
    /// Record a gauge value with the given key
    fn gauge(&self, key: &str, value: T) -> MetricResult<Gauge> {
        self.gauge_with_tags(key, value).try_send()
    }

    /// Record a gauge value with the given key and return a `MetricBuilder`
    /// that can be used to add tags to the metric.
    fn gauge_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Gauge>;
}

/// Trait for recording meter values.
///
/// Meter values measure the rate at which events occur. These rates are
/// determined by the server, the client simply indicates when they happen.
/// Meters can be thought of as increment-only counters. Examples include
/// things like number of requests handled or number of times something is
/// flushed to disk.
///
/// The following types are valid for meters:
/// * `u64`
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
pub trait Metered<T>
where
    T: ToMeterValue,
{
    /// Record a meter value with the given key
    fn meter(&self, key: &str, value: T) -> MetricResult<Meter> {
        self.meter_with_tags(key, value).try_send()
    }

    /// Record a meter value with the given key and return a `MetricBuilder`
    /// that can be used to add tags to the metric.
    fn meter_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Meter>;
}

/// Trait for recording histogram values.
///
/// Histogram values are positive values that can represent anything, whose
/// statistical distribution is calculated by the server. The values can be
/// timings, amount of some resource consumed, size of HTTP responses in
/// some application, etc. Histograms can be thought of as a more general
/// form of timers. `Duration` values are converted to nanoseconds before
/// being emitted.
///
/// The following types are valid for histograms:
/// * `u64`
/// * `f64`
/// * `Duration`
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that tags and histograms are a
/// [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) extension to
/// Statsd and may not be supported by your server.
pub trait Histogrammed<T>
where
    T: ToHistogramValue,
{
    /// Record a single histogram value with the given key
    fn histogram(&self, key: &str, value: T) -> MetricResult<Histogram> {
        self.histogram_with_tags(key, value).try_send()
    }

    /// Record a single histogram value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn histogram_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Histogram>;
}

/// Trait for recording distribution values.
///
/// Similar to histograms, but applies globally. A distribution can be used to
/// instrument logical objects, like services, independently from the underlying
/// hosts.
///
/// The following types are valid for distributions:
/// * `u64`
/// * `f64`
///
/// See the [Datadog docs](https://docs.datadoghq.com/developers/metrics/types/?tab=distribution#definition)
/// for more information.
///
/// Note that tags and distributions are a
/// [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) extension to
/// Statsd and may not be supported by your server.
pub trait Distributed<T>
where
    T: ToDistributionValue,
{
    /// Record a single distribution value with the given key
    fn distribution(&self, key: &str, value: T) -> MetricResult<Distribution> {
        self.distribution_with_tags(key, value).try_send()
    }

    /// Record a single distribution value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn distribution_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Distribution>;
}

/// Trait for recording set values.
///
/// Sets count the number of unique elements in a group. You can use them to,
/// for example, count the unique visitors to your site.
///
/// The following types are valid for sets:
/// * `i64`
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
pub trait Setted<T>
where
    T: ToSetValue,
{
    /// Record a single set value with the given key
    fn set(&self, key: &str, value: T) -> MetricResult<Set> {
        self.set_with_tags(key, value).try_send()
    }

    /// Record a single set value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn set_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Set>;
}

/// Trait that encompasses all other traits for sending metrics.
///
/// If you wish to use `StatsdClient` with a generic type or place a
/// `StatsdClient` instance behind a pointer (such as a `Box`) this will allow
/// you to reference all the implemented methods for recording metrics, while
/// using a single trait. An example of this is shown below.
///
/// ```
/// use std::time::Duration;
/// use cadence::{MetricClient, StatsdClient, NopMetricSink};
///
/// let client: Box<dyn MetricClient> = Box::new(StatsdClient::from_sink(
///     "prefix", NopMetricSink));
///
/// client.count("some.counter", 1).unwrap();
/// client.time("some.timer", 42).unwrap();
/// client.time("some.timer", Duration::from_millis(42)).unwrap();
/// client.time("some.timer", vec![42]).unwrap();
/// client.time("some.timer", vec![Duration::from_millis(42)]).unwrap();
/// client.gauge("some.gauge", 8).unwrap();
/// client.meter("some.meter", 13).unwrap();
/// client.histogram("some.histogram", 4).unwrap();
/// client.histogram("some.histogram", Duration::from_nanos(4)).unwrap();
/// client.histogram("some.histogram", vec![4]).unwrap();
/// client.histogram("some.histogram", vec![Duration::from_nanos(4)]).unwrap();
/// client.distribution("some.distribution", 4).unwrap();
/// client.distribution("some.distribution", vec![4]).unwrap();
/// client.set("some.set", 5).unwrap();
/// ```
pub trait MetricClient:
    Counted<i64>
    + CountedExt
    + Timed<u64>
    + Timed<Duration>
    + Timed<Vec<u64>>
    + Timed<Vec<Duration>>
    + Gauged<u64>
    + Gauged<f64>
    + Metered<u64>
    + Histogrammed<u64>
    + Histogrammed<f64>
    + Histogrammed<Duration>
    + Histogrammed<Vec<u64>>
    + Histogrammed<Vec<f64>>
    + Histogrammed<Vec<Duration>>
    + Distributed<u64>
    + Distributed<f64>
    + Distributed<Vec<u64>>
    + Distributed<Vec<f64>>
    + Setted<i64>
{
}

/// Typically internal client methods for sending metrics and handling errors.
///
/// This trait exposes methods of the client that would normally be internal
/// but may be useful for consumers of the library to extend it in unforseen
/// ways. Most consumers of the library shouldn't need to make use of this
/// extension point.
///
/// This trait is not exposed in the `prelude` module since it isn't required
/// to use the client for sending metrics. It is only exposed in the `ext`
/// module which is used to encompass advanced extension points for the library.
///
/// NOTE: This is a sealed trait and so it cannot be implemented outside of the
/// library.
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
pub trait MetricBackend: Sealed {
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
///     .with_tag("environment", "production")
///     .with_tag_value("rust")
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
    errors: Box<dyn Fn(MetricError) + Sync + Send + RefUnwindSafe>,
    tags: Vec<(Option<String>, String)>,
}

impl StatsdClientBuilder {
    // Set the required fields and defaults for optional fields
    fn new<T>(prefix: &str, sink: T) -> Self
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        StatsdClientBuilder {
            // required
            prefix: Self::formatted_prefix(prefix),
            sink: Box::new(sink),

            // optional with defaults
            errors: Box::new(nop_error_handler),
            tags: Vec::new(),
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
        F: Fn(MetricError) + Sync + Send + RefUnwindSafe + 'static,
    {
        self.errors = Box::new(errors);
        self
    }

    /// Add a default tag with key and value to every metric published by the
    /// built [StatsdClient].
    pub fn with_tag<K, V>(mut self, key: K, value: V) -> Self
    where
        K: ToString,
        V: ToString,
    {
        self.tags.push((Some(key.to_string()), value.to_string()));
        self
    }

    /// Add a default tag with only a value to every metric published by the built
    /// [StatsdClient].
    pub fn with_tag_value<K>(mut self, value: K) -> Self
    where
        K: ToString,
    {
        self.tags.push((None, value.to_string()));
        self
    }

    /// Construct a new `StatsdClient` instance based on current settings.
    pub fn build(self) -> StatsdClient {
        StatsdClient::from_builder(self)
    }

    fn formatted_prefix(prefix: &str) -> String {
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
/// * `Distributed` for emitting distribution values.
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
/// `Sync`). An example of how to use the client in a multithreaded environment
/// is given below.
///
/// In the following example, we create a struct `MyRequestHandler` that has a
/// single method that spawns a thread to do some work and emit a metric.
///
/// ## Wrapping With An `Arc`
///
/// In order to share a client between multiple threads, you'll need to wrap it
/// with an atomic reference counting pointer (`std::sync::Arc`). You should refer
/// to the client by the trait of all its methods for recording metrics
/// (`MetricClient`) as well as the `Send` and `Sync` traits since the idea is to
/// share this between threads.
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
///     metrics: Arc<dyn MetricClient + Send + Sync + RefUnwindSafe>,
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
///             metric_ref.count("request.handler", 1);
///         });
///
///         Ok(())
///     }
/// }
/// ```
pub struct StatsdClient {
    prefix: String,
    sink: Box<dyn MetricSink + Sync + Send + RefUnwindSafe>,
    errors: Box<dyn Fn(MetricError) + Sync + Send + RefUnwindSafe>,
    tags: Vec<(Option<String>, String)>,
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

    /// Flush the underlying metric sink.
    ///
    /// This is helpful for when you'd like to buffer metrics
    /// but still want strong control over when to emit them.
    /// For example, you are using a BufferedUdpMetricSink and
    /// have just emitted some time-sensitive metrics, but you
    /// aren't sure if the buffer is full or not. Thus, you can
    /// use `flush` to force the sink to flush your metrics now.
    ///
    /// # Buffered UDP Socket Example
    ///
    /// ```
    /// use std::net::UdpSocket;
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, BufferedUdpMetricSink, DEFAULT_PORT};
    ///
    /// let prefix = "my.stats";
    /// let host = ("127.0.0.1", DEFAULT_PORT);
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    ///
    /// let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    /// let client = StatsdClient::from_sink(prefix, sink);
    ///
    /// client.count("time-sensitive.keyA", 1);
    /// client.count("time-sensitive.keyB", 2);
    /// client.count("time-sensitive.keyC", 3);
    /// // Any number of time-sensitive metrics ... //
    /// client.flush();
    /// ```
    pub fn flush(&self) -> MetricResult<()> {
        Ok(self.sink.flush()?)
    }

    // Create a new StatsdClient by consuming the builder
    fn from_builder(builder: StatsdClientBuilder) -> Self {
        StatsdClient {
            prefix: builder.prefix,
            sink: builder.sink,
            errors: builder.errors,
            tags: builder.tags,
        }
    }

    fn tags(&self) -> impl IntoIterator<Item = (Option<&str>, &str)> {
        self.tags.iter().map(|(k, v)| (k.as_deref(), v.as_str()))
    }
}

impl Sealed for StatsdClient {}

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
            "StatsdClient {{ prefix: {:?}, sink: ..., errors: ..., tags: {:?} }}",
            self.prefix, self.tags,
        )
    }
}

impl<T> Counted<T> for StatsdClient
where
    T: ToCounterValue,
{
    fn count_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Counter> {
        match value.try_to_value() {
            Ok(v) => {
                MetricBuilder::from_fmt(MetricFormatter::counter(&self.prefix, key, v), self).with_tags(self.tags())
            }
            Err(e) => MetricBuilder::from_error(e, self),
        }
    }
}

impl CountedExt for StatsdClient {}

impl<T> Timed<T> for StatsdClient
where
    T: ToTimerValue,
{
    fn time_with_tags<'a>(&'a self, key: &'a str, time: T) -> MetricBuilder<'_, '_, Timer> {
        match time.try_to_value() {
            Ok(v) => MetricBuilder::from_fmt(MetricFormatter::timer(&self.prefix, key, v), self).with_tags(self.tags()),
            Err(e) => MetricBuilder::from_error(e, self),
        }
    }
}

impl<T> Gauged<T> for StatsdClient
where
    T: ToGaugeValue,
{
    fn gauge_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Gauge> {
        match value.try_to_value() {
            Ok(v) => MetricBuilder::from_fmt(MetricFormatter::gauge(&self.prefix, key, v), self).with_tags(self.tags()),
            Err(e) => MetricBuilder::from_error(e, self),
        }
    }
}

impl<T> Metered<T> for StatsdClient
where
    T: ToMeterValue,
{
    fn meter_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Meter> {
        match value.try_to_value() {
            Ok(v) => MetricBuilder::from_fmt(MetricFormatter::meter(&self.prefix, key, v), self).with_tags(self.tags()),
            Err(e) => MetricBuilder::from_error(e, self),
        }
    }
}

impl<T> Histogrammed<T> for StatsdClient
where
    T: ToHistogramValue,
{
    fn histogram_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Histogram> {
        match value.try_to_value() {
            Ok(v) => {
                MetricBuilder::from_fmt(MetricFormatter::histogram(&self.prefix, key, v), self).with_tags(self.tags())
            }
            Err(e) => MetricBuilder::from_error(e, self),
        }
    }
}

impl<T> Distributed<T> for StatsdClient
where
    T: ToDistributionValue,
{
    fn distribution_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Distribution> {
        match value.try_to_value() {
            Ok(v) => MetricBuilder::from_fmt(MetricFormatter::distribution(&self.prefix, key, v), self)
                .with_tags(self.tags()),
            Err(e) => MetricBuilder::from_error(e, self),
        }
    }
}

impl<T> Setted<T> for StatsdClient
where
    T: ToSetValue,
{
    fn set_with_tags<'a>(&'a self, key: &'a str, value: T) -> MetricBuilder<'_, '_, Set> {
        match value.try_to_value() {
            Ok(v) => MetricBuilder::from_fmt(MetricFormatter::set(&self.prefix, key, v), self).with_tags(self.tags()),
            Err(e) => MetricBuilder::from_error(e, self),
        }
    }
}

impl MetricClient for StatsdClient {}

#[allow(clippy::needless_pass_by_value)]
fn nop_error_handler(_err: MetricError) {
    // nothing
}

#[cfg(test)]
mod tests {
    use super::{
        Counted, CountedExt, Distributed, Gauged, Histogrammed, Metered, MetricClient, Setted, StatsdClient, Timed,
    };
    use crate::sinks::{MetricSink, NopMetricSink, QueuingMetricSink, SpyMetricSink};
    use crate::types::{ErrorKind, Metric, MetricError};
    use crate::StatsdClientBuilder;
    use std::io;
    use std::panic::RefUnwindSafe;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use std::u64;

    #[test]
    fn test_statsd_client_empty_prefix() {
        let client = StatsdClient::from_sink("", NopMetricSink);
        let res = client.count("some.method", 1);

        assert_eq!("some.method:1|c", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_merging_default_tags_with_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("hello", "world")
            .with_tag_value("production")
            .build();
        let res = client
            .count_with_tags("some.counter", 3)
            .with_tag("foo", "bar")
            .with_tag_value("fizz")
            .with_tag("bucket", "123")
            .try_send();

        assert_eq!(
            "prefix.some.counter:3|c|#hello:world,production,foo:bar,fizz,bucket:123",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_count_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .count_with_tags("some.counter", 3)
            .with_tag("foo", "bar")
            .try_send();

        assert_eq!("prefix.some.counter:3|c|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_count_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("hello", "world")
            .build();
        let res = client.count_with_tags("some.counter", 3).try_send();

        assert_eq!("prefix.some.counter:3|c|#hello:world", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_incr_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.incr_with_tags("some.counter").with_tag("foo", "bar").try_send();

        assert_eq!("prefix.some.counter:1|c|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_incr_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.incr_with_tags("some.counter").try_send();

        assert_eq!("prefix.some.counter:1|c|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_decr_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.decr_with_tags("some.counter").with_tag("foo", "bar").try_send();

        assert_eq!("prefix.some.counter:-1|c|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_decr_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.decr_with_tags("some.counter").try_send();

        assert_eq!("prefix.some.counter:-1|c|#foo:bar", res.unwrap().as_metric_str());
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
    fn test_statsd_client_gauge_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.gauge_with_tags("some.gauge", 4).try_send();

        assert_eq!("prefix.some.gauge:4|g|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_time_duration() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.time("key", Duration::from_millis(157));

        assert_eq!("prefix.key:157|ms", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_time_multiple_durations() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let durations = vec![
            Duration::from_millis(157),
            Duration::from_millis(158),
            Duration::from_millis(159),
        ];
        let res = client.time("key", durations);

        assert_eq!("prefix.key:157:158:159|ms", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_time_duration_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.time("key", Duration::from_secs(u64::MAX));

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind())
    }

    #[test]
    fn test_statsd_client_time_multiple_durations_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let durations = vec![
            Duration::from_millis(157),
            Duration::from_secs(u64::MAX),
            Duration::from_millis(159),
        ];
        let res = client.time("key", durations);

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind())
    }

    #[test]
    fn test_statsd_client_time_duration_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .time_with_tags("key", Duration::from_millis(157))
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .try_send();

        assert_eq!("prefix.key:157|ms|#foo:bar,quux", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_time_duration_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.time("key", Duration::from_millis(157));

        assert_eq!("prefix.key:157|ms|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_time_multiple_durations_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let durations = vec![
            Duration::from_millis(157),
            Duration::from_millis(158),
            Duration::from_millis(159),
        ];
        let res = client
            .time_with_tags("key", durations)
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .try_send();

        assert_eq!("prefix.key:157:158:159|ms|#foo:bar,quux", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_time_duration_with_tags_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .time_with_tags("key", Duration::from_secs(u64::MAX))
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .try_send();

        assert!(res.is_err());
        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_time_multiple_durations_with_tags_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let durations = vec![
            Duration::from_millis(157),
            Duration::from_secs(u64::MAX),
            Duration::from_millis(159),
        ];
        let res = client
            .time_with_tags("key", durations)
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .try_send();

        assert!(res.is_err());
        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_meter_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .meter_with_tags("some.meter", 64)
            .with_tag("segment", "142")
            .with_tag_value("beta")
            .try_send();

        assert_eq!("prefix.some.meter:64|m|#segment:142,beta", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_meter_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.meter_with_tags("some.meter", 64).try_send();

        assert_eq!("prefix.some.meter:64|m|#foo:bar", res.unwrap().as_metric_str());
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
    fn test_statsd_client_histogram_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.histogram_with_tags("some.histo", 27).try_send();

        assert_eq!("prefix.some.histo:27|h|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_histogram_with_multiple_values() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.histogram_with_tags("some.histo", vec![27, 28, 29]).try_send();

        assert_eq!("prefix.some.histo:27:28:29|h", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_histogram_duration() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.histogram("key", Duration::from_nanos(210));

        assert_eq!("prefix.key:210|h", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_histogram_multiple_durations() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let durations = vec![
            Duration::from_nanos(210),
            Duration::from_nanos(211),
            Duration::from_nanos(212),
        ];
        let res = client.histogram("key", durations);

        assert_eq!("prefix.key:210:211:212|h", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_histogram_duration_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client.histogram("key", Duration::from_secs(u64::MAX));

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_histogram_multiple_durations_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let durations = vec![
            Duration::from_nanos(210),
            Duration::from_secs(u64::MAX),
            Duration::from_nanos(212),
        ];

        let res = client.histogram("key", durations);

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_histogram_duration_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .histogram_with_tags("key", Duration::from_nanos(4096))
            .with_tag("foo", "bar")
            .with_tag_value("beta")
            .try_send();

        assert_eq!("prefix.key:4096|h|#foo:bar,beta", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_histogram_duration_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.histogram_with_tags("key", Duration::from_nanos(4096)).try_send();

        assert_eq!("prefix.key:4096|h|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_histogram_duration_with_tags_with_overflow() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .histogram_with_tags("key", Duration::from_millis(u64::MAX))
            .with_tag("foo", "bar")
            .with_tag_value("beta")
            .try_send();

        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    #[test]
    fn test_statsd_client_distribution_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .distribution_with_tags("some.distr", 27)
            .with_tag("host", "www03.example.com")
            .with_tag_value("rc1")
            .try_send();

        assert_eq!(
            "prefix.some.distr:27|d|#host:www03.example.com,rc1",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_distribution_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client
            .distribution_with_tags("some.distr", 27)
            .with_tag("host", "www03.example.com")
            .with_tag_value("rc1")
            .try_send();

        assert_eq!(
            "prefix.some.distr:27|d|#foo:bar,host:www03.example.com,rc1",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_distribution_multiple_values_with_tags() {
        let client = StatsdClient::from_sink("prefix", NopMetricSink);
        let res = client
            .distribution_with_tags("some.distr", vec![27, 28, 29])
            .with_tag("host", "www03.example.com")
            .with_tag_value("rc1")
            .try_send();

        assert_eq!(
            "prefix.some.distr:27:28:29|d|#host:www03.example.com,rc1",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_set_with_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.set_with_tags("some.set", 3).with_tag("foo", "bar").try_send();

        assert_eq!("myapp.some.set:3|s|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_set_with_default_tags() {
        let client = StatsdClientBuilder::new("prefix", NopMetricSink)
            .with_tag("foo", "bar")
            .build();
        let res = client.set_with_tags("some.set", 3).try_send();

        assert_eq!("prefix.some.set:3|s|#foo:bar", res.unwrap().as_metric_str());
    }

    #[test]
    fn test_statsd_client_with_tags_send_success() {
        let (rx, sink) = SpyMetricSink::new();
        let client = StatsdClient::from_sink("prefix", sink);

        client.count_with_tags("some.key", 1).with_tag("test", "a").send();
        let sent = rx.recv().unwrap();

        assert_eq!("prefix.some.key:1|c|#test:a", String::from_utf8(sent).unwrap());
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
        let count_ref = count.clone();

        let handler = move |_err: MetricError| {
            count_ref.fetch_add(1, Ordering::Release);
        };

        let client = StatsdClient::builder("prefix", ErrorSink)
            .with_error_handler(handler)
            .build();

        client.count_with_tags("some.key", 1).with_tag("tier", "web").send();

        assert_eq!(1, count.load(Ordering::Acquire));
    }

    // The following tests really just ensure that we've actually
    // implemented all the traits we're supposed to correctly. If
    // we hadn't, this wouldn't compile.

    #[test]
    fn test_statsd_client_as_counted() {
        let client: Box<dyn Counted<i64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.count("some.counter", 5).unwrap();
    }

    #[test]
    fn test_statsd_client_as_countedext() {
        let client: Box<dyn CountedExt> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.incr("some.counter").unwrap();
    }

    #[test]
    fn test_statsd_client_as_timed_u64() {
        let client: Box<dyn Timed<u64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.time("some.timer", 20).unwrap();
    }

    #[test]
    fn test_statsd_client_as_timed_duration() {
        let client: Box<dyn Timed<Duration>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.time("some.timer", Duration::from_millis(20)).unwrap();
    }

    #[test]
    fn test_statsd_client_as_timed_packed_duration() {
        let client: Box<dyn Timed<Vec<Duration>>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));
        let durations = vec![Duration::from_millis(20), Duration::from_millis(21)];

        client.time("some.timer", durations).unwrap();
    }

    #[test]
    fn test_statsd_client_as_gauged_u64() {
        let client: Box<dyn Gauged<u64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.gauge("some.gauge", 32).unwrap();
    }

    #[test]
    fn test_statsd_client_as_gauged_f64() {
        let client: Box<dyn Gauged<f64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.gauge("some.gauge", 3.2).unwrap();
    }

    #[test]
    fn test_statsd_client_as_metered() {
        let client: Box<dyn Metered<u64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.meter("some.meter", 9).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed_u64() {
        let client: Box<dyn Histogrammed<u64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.histogram("some.histogram", 4).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed_packed_u64() {
        let client: Box<dyn Histogrammed<Vec<u64>>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.histogram("some.histogram", vec![4, 5, 6]).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed_f64() {
        let client: Box<dyn Histogrammed<f64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.histogram("some.histogram", 4.0).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed_packed_f64() {
        let client: Box<dyn Histogrammed<Vec<f64>>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.histogram("some.histogram", vec![4.0, 5.0, 6.0]).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed_duration() {
        let client: Box<dyn Histogrammed<Duration>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.histogram("some.histogram", Duration::from_nanos(4)).unwrap();
    }

    #[test]
    fn test_statsd_client_as_histogrammed_packed_duration() {
        let client: Box<dyn Histogrammed<Vec<Duration>>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));
        let durations = vec![Duration::from_nanos(4), Duration::from_nanos(5)];

        client.histogram("some.histogram", durations).unwrap();
    }

    #[test]
    fn test_statsd_client_as_distributed_u64() {
        let client: Box<dyn Distributed<u64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.distribution("some.distribution", 33).unwrap();
    }

    #[test]
    fn test_statsd_client_as_distributed_packed_u64() {
        let client: Box<dyn Distributed<Vec<u64>>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.distribution("some.distribution", vec![33, 34]).unwrap();
    }

    #[test]
    fn test_statsd_client_as_distributed_f64() {
        let client: Box<dyn Distributed<f64>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.distribution("some.distribution", 33.0).unwrap();
    }

    #[test]
    fn test_statsd_client_as_distributed_packed_f64() {
        let client: Box<dyn Distributed<Vec<f64>>> = Box::new(StatsdClient::from_sink("prefix", NopMetricSink));

        client.distribution("some.distribution", vec![33.0, 34.0]).unwrap();
    }

    #[test]
    fn test_statsd_client_as_setted() {
        let client: Box<dyn Setted<i64>> = Box::new(StatsdClient::from_sink("myapp", NopMetricSink));

        client.set("some.set", 5).unwrap();
    }

    #[test]
    fn test_statsd_client_as_thread_and_panic_safe() {
        let client: Box<dyn MetricClient + Send + Sync + RefUnwindSafe> = Box::new(StatsdClient::from_sink(
            "prefix",
            QueuingMetricSink::from(NopMetricSink),
        ));

        client.count("some.counter", 3).unwrap();
        client.time("some.timer", 198).unwrap();
        client.time("some.timer", Duration::from_millis(198)).unwrap();
        client.time("some.timer", vec![198]).unwrap();
        client.time("some.timer", vec![Duration::from_millis(198)]).unwrap();
        client.gauge("some.gauge", 4).unwrap();
        client.gauge("some.gauge", 4.0).unwrap();
        client.meter("some.meter", 29).unwrap();
        client.histogram("some.histogram", 32).unwrap();
        client.histogram("some.histogram", 32.0).unwrap();
        client.histogram("some.histogram", Duration::from_nanos(32)).unwrap();
        client.histogram("some.histogram", vec![32]).unwrap();
        client.histogram("some.histogram", vec![32.0]).unwrap();
        client
            .histogram("some.histogram", vec![Duration::from_nanos(32)])
            .unwrap();
        client.distribution("some.distribution", 248).unwrap();
        client.distribution("some.distribution", 248.0).unwrap();
        client.distribution("some.distribution", vec![248]).unwrap();
        client.distribution("some.distribution", vec![248.0]).unwrap();
        client.set("some.set", 5).unwrap();
    }
}

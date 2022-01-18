// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2018 Philip Jenvey <pjenvey@mozilla.com>
// Copyright 2018-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::client::{MetricBackend, StatsdClient};
use crate::types::{Metric, MetricError, MetricResult};
use std::fmt::{self, Write};
use std::marker::PhantomData;

/// Type of metric that knows how to display itself
#[derive(Debug, Clone, Copy)]
enum MetricType {
    Counter,
    Timer,
    Gauge,
    Meter,
    Histogram,
    Set,
    Distribution,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MetricType::Counter => "c".fmt(f),
            MetricType::Timer => "ms".fmt(f),
            MetricType::Gauge => "g".fmt(f),
            MetricType::Meter => "m".fmt(f),
            MetricType::Histogram => "h".fmt(f),
            MetricType::Set => "s".fmt(f),
            MetricType::Distribution => "d".fmt(f),
        }
    }
}

/// Holder for primitive metric values that knows how to display itself
///
/// This struct is internal to how various types that are valid for each type
/// of metric (e.g. types for which `ToCounterValue`, `ToTimerValue`, etc) are
/// implemented but is exposed for documentation purposes and advanced use cases.
///
/// Typical use of Cadence shouldn't require interacting with this type.
#[derive(Debug, Clone, Copy)]
pub enum MetricValue {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MetricValue::Signed(v) => v.fmt(f),
            MetricValue::Unsigned(v) => v.fmt(f),
            MetricValue::Float(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MetricFormatter<'a> {
    prefix: &'a str,
    key: &'a str,
    val: Vec<MetricValue>,
    type_: MetricType,
    tags: Vec<(Option<&'a str>, &'a str)>,
    base_size: usize,
    kv_size: usize,
}

impl<'a> MetricFormatter<'a> {
    const TAG_PREFIX: &'static str = "|#";

    pub(crate) fn counter(prefix: &'a str, key: &'a str, val: Vec<MetricValue>) -> Self {
        Self::from_val(prefix, key, val, MetricType::Counter)
    }

    pub(crate) fn timer(prefix: &'a str, key: &'a str, val: Vec<MetricValue>) -> Self {
        Self::from_val(prefix, key, val, MetricType::Timer)
    }

    pub(crate) fn gauge(prefix: &'a str, key: &'a str, val: Vec<MetricValue>) -> Self {
        Self::from_val(prefix, key, val, MetricType::Gauge)
    }

    pub(crate) fn meter(prefix: &'a str, key: &'a str, val: Vec<MetricValue>) -> Self {
        Self::from_val(prefix, key, val, MetricType::Meter)
    }

    pub(crate) fn histogram(prefix: &'a str, key: &'a str, val: Vec<MetricValue>) -> Self {
        Self::from_val(prefix, key, val, MetricType::Histogram)
    }

    pub(crate) fn distribution(prefix: &'a str, key: &'a str, val: Vec<MetricValue>) -> Self {
        Self::from_val(prefix, key, val, MetricType::Distribution)
    }

    pub(crate) fn set(prefix: &'a str, key: &'a str, val: Vec<MetricValue>) -> Self {
        Self::from_val(prefix, key, val, MetricType::Set)
    }

    #[rustfmt::skip]
    fn from_val(prefix: &'a str, key: &'a str, val: Vec<MetricValue>, type_: MetricType) -> Self {
        MetricFormatter {
            prefix,
            key,
            type_,
            val,
            tags: Vec::new(),
            // keep track of the number of bytes we expect to use for both the key-value
            // part of the tags for this metric as well as the base metric (name, value,
            // and type). incrementing these counters when tags are added saves us from
            // having to loop through the tags to count the expected number of bytes to
            // allocate.
            kv_size: 0,
            base_size: prefix.len() + key.len() + 1 /* : */ + 10 /* value */ + 1 /* | */ + 2, /* type */
        }
    }

    fn with_tag(&mut self, key: &'a str, value: &'a str) {
        self.tags.push((Some(key), value));
        self.kv_size += key.len() + 1 /* : */ + value.len();
    }

    fn with_tag_value(&mut self, value: &'a str) {
        self.tags.push((None, value));
        self.kv_size += value.len();
    }

    fn write_value(&self) -> String {
        let mut out = String::new();

        for (i, value) in self.val.iter().enumerate() {
            if i > 0 {
                out.push(':');
            }
            out.push_str(format!("{}", value).as_str());
        }

        out
    }

    fn write_base_metric(&self, out: &mut String) {
        let _ = write!(out, "{}{}:{}|{}", self.prefix, self.key, self.write_value(), self.type_);
    }

    fn write_tags(&self, out: &mut String) {
        if !self.tags.is_empty() {
            out.push_str(Self::TAG_PREFIX);
            for (i, &(key, value)) in self.tags.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if let Some(key) = key {
                    out.push_str(key);
                    out.push(':');
                }
                out.push_str(value);
            }
        }
    }

    fn tag_size_hint(&self) -> usize {
        if self.tags.is_empty() {
            return 0;
        }

        // prefix, keys and values, commas
        Self::TAG_PREFIX.len() + self.kv_size + self.tags.len() - 1
    }

    pub(crate) fn format(&self) -> String {
        let size_hint = self.base_size + self.tag_size_hint();
        let mut metric_string = String::with_capacity(size_hint);
        self.write_base_metric(&mut metric_string);
        self.write_tags(&mut metric_string);
        metric_string
    }
}

/// Internal state of a `MetricBuilder`
///
/// The builder can either be in the process of formatting a metric to send
/// via a client or it can be simply holding on to an error that it will be
/// dealt with when `.try_send()` or `.send()` is finally invoked.
#[derive(Debug)]
enum BuilderRepr<'m, 'c> {
    Success(MetricFormatter<'m>, &'c StatsdClient),
    Error(MetricError, &'c StatsdClient),
}

/// Builder for adding tags to in-progress metrics.
///
/// This builder adds tags, key-value pairs or just values, to a metric that
/// was previously constructed by a call to a method on `StatsdClient`. The
/// tags are added to metrics and sent via the client when `MetricBuilder::send()`
/// or `MetricBuilder::try_send()`is invoked. Any errors encountered constructing,
/// validating, or sending the metrics will be propagated and returned when those
/// methods are finally invoked.
///
/// Currently, only Datadog style tags are supported. For more information on the
/// exact format used, see the
/// [Datadog docs](https://docs.datadoghq.com/developers/dogstatsd/#datagram-format).
///
/// Adding tags to a metric via this builder will typically result in one or more
/// extra heap allocations.
///
/// NOTE: The only way to instantiate an instance of this builder is via methods in
/// in the `StatsdClient` client.
///
/// # Examples
///
/// ## `.try_send()`
///
/// An example of how the metric builder is used with a `StatsdClient` instance
/// is given below.
///
/// ```
/// use cadence::prelude::*;
/// use cadence::{StatsdClient, NopMetricSink, Metric};
///
/// let client = StatsdClient::from_sink("some.prefix", NopMetricSink);
/// let res = client.count_with_tags("some.key", 1)
///    .with_tag("host", "app11.example.com")
///    .with_tag("segment", "23")
///    .with_tag_value("beta")
///    .try_send();
///
/// assert_eq!(
///     concat!(
///         "some.prefix.some.key:1|c|#",
///         "host:app11.example.com,",
///         "segment:23,",
///         "beta"
///     ),
///     res.unwrap().as_metric_str()
/// );
/// ```
///
/// In this example, two key-value tags and one value tag are added to the
/// metric before it is finally sent to the Statsd server.
///
/// ## `.send()`
///
/// An example of how the metric builder is used with a `StatsdClient` instance
/// when using the "quiet" method is given below.
///
/// ```
/// use cadence::prelude::*;
/// use cadence::{StatsdClient, NopMetricSink, Metric};
///
/// let client = StatsdClient::builder("some.prefix", NopMetricSink)
///     .with_error_handler(|e| eprintln!("metric error: {}", e))
///     .build();
/// client.count_with_tags("some.key", 1)
///    .with_tag("host", "app11.example.com")
///    .with_tag("segment", "23")
///    .with_tag_value("beta")
///    .send();
/// ```
///
/// Note that nothing is returned from the `.send()` method. Any errors encountered
/// in this case will be passed to the error handler we registered.
#[must_use = "Did you forget to call .send() after adding tags?"]
#[derive(Debug)]
pub struct MetricBuilder<'m, 'c, T>
where
    T: Metric + From<String>,
{
    repr: BuilderRepr<'m, 'c>,
    type_: PhantomData<T>,
}

impl<'m, 'c, T> MetricBuilder<'m, 'c, T>
where
    T: Metric + From<String>,
{
    pub(crate) fn from_fmt(formatter: MetricFormatter<'m>, client: &'c StatsdClient) -> Self {
        MetricBuilder {
            repr: BuilderRepr::Success(formatter, client),
            type_: PhantomData,
        }
    }

    pub(crate) fn from_error(err: MetricError, client: &'c StatsdClient) -> Self {
        MetricBuilder {
            repr: BuilderRepr::Error(err, client),
            type_: PhantomData,
        }
    }

    /// Add a key-value tag to this metric.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, Metric};
    ///
    /// let client = StatsdClient::from_sink("some.prefix", NopMetricSink);
    /// let res = client.count_with_tags("some.key", 1)
    ///    .with_tag("user", "authenticated")
    ///    .try_send();
    ///
    /// assert_eq!(
    ///    "some.prefix.some.key:1|c|#user:authenticated",
    ///    res.unwrap().as_metric_str()
    /// );
    /// ```
    pub fn with_tag(mut self, key: &'m str, value: &'m str) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_tag(key, value);
        }
        self
    }

    /// Add a value tag to this metric.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, Metric};
    ///
    /// let client = StatsdClient::from_sink("some.prefix", NopMetricSink);
    /// let res = client.count_with_tags("some.key", 4)
    ///    .with_tag_value("beta-testing")
    ///    .try_send();
    ///
    /// assert_eq!(
    ///    "some.prefix.some.key:4|c|#beta-testing",
    ///    res.unwrap().as_metric_str()
    /// );
    /// ```
    pub fn with_tag_value(mut self, value: &'m str) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_tag_value(value);
        }
        self
    }

    /// Send a metric using the client that created this builder.
    ///
    /// Note that the builder is consumed by this method and thus `.try_send()`
    /// can only be called a single time per builder.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, Metric};
    ///
    /// let client = StatsdClient::from_sink("some.prefix", NopMetricSink);
    /// let res = client.gauge_with_tags("some.key", 7)
    ///    .with_tag("test-segment", "12345")
    ///    .try_send();
    ///
    /// assert_eq!(
    ///    "some.prefix.some.key:7|g|#test-segment:12345",
    ///    res.unwrap().as_metric_str()
    /// );
    /// ```
    pub fn try_send(self) -> MetricResult<T> {
        match self.repr {
            BuilderRepr::Error(err, _) => Err(err),
            BuilderRepr::Success(ref formatter, client) => {
                let metric = T::from(formatter.format());
                client.send_metric(&metric)?;
                Ok(metric)
            }
        }
    }

    /// Send a metric using the client that created this builder, discarding
    /// successful results and invoking a custom handler for error results.
    ///
    /// By default, if no handler is given, a "no-op" handler is used that
    /// simply discards all errors. If this isn't desired, a custom handler
    /// should be supplied when creating a new `StatsdClient` instance.
    ///
    /// Note that the builder is consumed by this method and thus `.send()`
    /// can only be called a single time per builder.
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
    pub fn send(self) {
        match self.repr {
            BuilderRepr::Error(err, client) => client.consume_error(err),
            BuilderRepr::Success(_, client) => {
                if let Err(e) = self.try_send() {
                    client.consume_error(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MetricBuilder, MetricFormatter, MetricValue};
    use crate::client::StatsdClient;
    use crate::sinks::NopMetricSink;
    use crate::test::ErrorMetricSink;
    use crate::types::Counter;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_metric_formatter_tag_size_hint_no_tags() {
        let fmt = MetricFormatter::counter("prefix.", "some.key", vec![MetricValue::Signed(1)]);
        assert_eq!(0, fmt.tag_size_hint());
    }

    #[test]
    fn test_metric_formatter_tag_size_hint_value() {
        let mut fmt = MetricFormatter::counter("prefix.", "some.key", vec![MetricValue::Signed(1)]);
        fmt.with_tag_value("test");

        assert_eq!(6, fmt.tag_size_hint());
    }

    #[test]
    fn test_metric_formatter_tag_size_hint_key_value() {
        let mut fmt = MetricFormatter::counter("prefix.", "some.key", vec![MetricValue::Signed(1)]);
        fmt.with_tag("host", "web");
        fmt.with_tag("user", "123");

        assert_eq!(19, fmt.tag_size_hint());
    }

    #[test]
    fn test_metric_formatter_counter_no_tags() {
        let fmt = MetricFormatter::counter("prefix.", "some.key", vec![MetricValue::Signed(4)]);
        assert_eq!("prefix.some.key:4|c", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_counter_with_tags() {
        let mut fmt = MetricFormatter::counter("prefix.", "some.key", vec![MetricValue::Signed(4)]);
        fmt.with_tag("host", "app03.example.com");
        fmt.with_tag("bucket", "2");
        fmt.with_tag_value("beta");

        assert_eq!(
            "prefix.some.key:4|c|#host:app03.example.com,bucket:2,beta",
            &fmt.format()
        );
    }

    #[test]
    fn test_metric_formatter_timer_no_tags() {
        let fmt = MetricFormatter::timer("prefix.", "some.method", vec![MetricValue::Unsigned(21)]);

        assert_eq!("prefix.some.method:21|ms", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_timer_with_tags() {
        let mut fmt = MetricFormatter::timer("prefix.", "some.method", vec![MetricValue::Unsigned(21)]);
        fmt.with_tag("app", "metrics");
        fmt.with_tag_value("async");

        assert_eq!("prefix.some.method:21|ms|#app:metrics,async", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_gauge_no_tags() {
        let fmt = MetricFormatter::gauge("prefix.", "num.failures", vec![MetricValue::Unsigned(7)]);

        assert_eq!("prefix.num.failures:7|g", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_gauge_with_tags() {
        let mut fmt = MetricFormatter::gauge("prefix.", "num.failures", vec![MetricValue::Unsigned(7)]);
        fmt.with_tag("window", "300");
        fmt.with_tag_value("best-effort");

        assert_eq!("prefix.num.failures:7|g|#window:300,best-effort", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_meter_no_tags() {
        let fmt = MetricFormatter::meter("prefix.", "user.logins", vec![MetricValue::Unsigned(3)]);

        assert_eq!("prefix.user.logins:3|m", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_meter_with_tags() {
        let mut fmt = MetricFormatter::meter("prefix.", "user.logins", vec![MetricValue::Unsigned(3)]);
        fmt.with_tag("user-type", "verified");
        fmt.with_tag_value("bucket1");

        assert_eq!("prefix.user.logins:3|m|#user-type:verified,bucket1", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_histogram_no_tags() {
        let fmt = MetricFormatter::histogram("prefix.", "num.results", vec![MetricValue::Unsigned(44)]);

        assert_eq!("prefix.num.results:44|h", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_histogram_with_tags() {
        let mut fmt = MetricFormatter::histogram("prefix.", "num.results", vec![MetricValue::Unsigned(44)]);
        fmt.with_tag("user-type", "authenticated");
        fmt.with_tag_value("source=search");

        assert_eq!(
            "prefix.num.results:44|h|#user-type:authenticated,source=search",
            &fmt.format()
        );
    }

    #[test]
    fn test_metric_formatter_set_no_tags() {
        let fmt = MetricFormatter::set("prefix.", "users.uniques", vec![MetricValue::Signed(44)]);

        assert_eq!("prefix.users.uniques:44|s", &fmt.format());
    }

    #[test]
    fn test_metric_formatter_set_with_tags() {
        let mut fmt = MetricFormatter::set("prefix.", "users.uniques", vec![MetricValue::Signed(44)]);
        fmt.with_tag("user-type", "authenticated");
        fmt.with_tag_value("source=search");

        assert_eq!(
            concat!(
                "prefix.users.uniques:44|s|#",
                "user-type:authenticated,",
                "source=search"
            ),
            &fmt.format()
        );
    }

    #[test]
    fn test_metric_builder_send_success() {
        let fmt = MetricFormatter::counter("prefix.", "some.counter", vec![MetricValue::Signed(11)]);
        let client = StatsdClient::builder("prefix.", NopMetricSink)
            .with_error_handler(|e| {
                panic!("unexpected error sending metric: {}", e);
            })
            .build();

        // if the send failed the test would have called the error handler and panicked
        let builder: MetricBuilder<'_, '_, Counter> = MetricBuilder::from_fmt(fmt, &client);
        builder.send();
    }

    #[test]
    fn test_metric_builder_send_error() {
        let errors = Arc::new(AtomicU64::new(0));
        let errors_ref = errors.clone();

        let fmt = MetricFormatter::counter("prefix.", "some.counter", vec![MetricValue::Signed(11)]);
        let client = StatsdClient::builder("prefix.", ErrorMetricSink::always())
            .with_error_handler(move |_e| {
                errors_ref.fetch_add(1, Ordering::Release);
            })
            .build();

        let builder: MetricBuilder<'_, '_, Counter> = MetricBuilder::from_fmt(fmt, &client);
        builder.send();

        assert_eq!(1, errors.load(Ordering::Acquire));
    }

    #[test]
    fn test_metric_builder_try_send_success() {
        let fmt = MetricFormatter::counter("prefix.", "some.counter", vec![MetricValue::Signed(11)]);
        let client = StatsdClient::from_sink("prefix.", NopMetricSink);

        let builder: MetricBuilder<'_, '_, Counter> = MetricBuilder::from_fmt(fmt, &client);
        let res = builder.try_send();

        assert!(res.is_ok(), "expected Ok result from try_send");
    }

    #[test]
    fn test_metric_builder_try_send_error() {
        let fmt = MetricFormatter::counter("prefix.", "some.counter", vec![MetricValue::Signed(11)]);
        let client = StatsdClient::from_sink("prefix.", ErrorMetricSink::always());

        let builder: MetricBuilder<'_, '_, Counter> = MetricBuilder::from_fmt(fmt, &client);
        let res = builder.try_send();

        assert!(res.is_err(), "expected Err result from try_send");
    }
}

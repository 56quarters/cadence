// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2018 Philip Jenvey <pjenvey@mozilla.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::{self, Write};
use std::marker::PhantomData;
use client::StatsdClient;
use types::{Metric, MetricResult};

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
enum MetricValue {
    Signed(i64),
    Unsigned(u64),
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetricValue::Signed(i) => i.fmt(f),
            MetricValue::Unsigned(i) => i.fmt(f),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone, Copy)]
enum MetricType {
    Counter,
    Timer,
    Gauge,
    Meter,
    Histogram,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetricType::Counter => "c".fmt(f),
            MetricType::Timer => "ms".fmt(f),
            MetricType::Gauge => "g".fmt(f),
            MetricType::Meter => "m".fmt(f),
            MetricType::Histogram => "h".fmt(f),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub(crate) struct MetricFormatter<'a, T>
where
    T: Metric + From<String>,
{
    metric: PhantomData<T>,
    prefix: &'a str,
    key: &'a str,
    val: MetricValue,
    type_: MetricType,
    tags: Option<Vec<(Option<&'a str>, &'a str)>>,
}

impl<'a, T> MetricFormatter<'a, T>
where
    T: Metric + From<String>,
{
    pub(crate) fn counter(prefix: &'a str, key: &'a str, val: i64) -> Self {
        Self::from_i64(prefix, key, val, MetricType::Counter)
    }

    pub(crate) fn timer(prefix: &'a str, key: &'a str, val: u64) -> Self {
        Self::from_u64(prefix, key, val, MetricType::Timer)
    }

    pub(crate) fn gauge(prefix: &'a str, key: &'a str, val: u64) -> Self {
        Self::from_u64(prefix, key, val, MetricType::Gauge)
    }

    pub(crate) fn meter(prefix: &'a str, key: &'a str, val: u64) -> Self {
        Self::from_u64(prefix, key, val, MetricType::Meter)
    }

    pub(crate) fn histogram(prefix: &'a str, key: &'a str, val: u64) -> Self {
        Self::from_u64(prefix, key, val, MetricType::Histogram)
    }

    fn from_u64(prefix: &'a str, key: &'a str, val: u64, type_: MetricType) -> Self {
        MetricFormatter {
            metric: PhantomData,
            prefix: prefix,
            key: key,
            val: MetricValue::Unsigned(val),
            type_: type_,
            tags: None,
        }
    }

    fn from_i64(prefix: &'a str, key: &'a str, val: i64, type_: MetricType) -> Self {
        MetricFormatter {
            metric: PhantomData,
            prefix: prefix,
            key: key,
            val: MetricValue::Signed(val),
            type_: type_,
            tags: None,
        }
    }

    fn with_tag(&mut self, key: &'a str, value: &'a str) {
        self.tags
            .get_or_insert_with(|| Vec::new())
            .push((Some(key), value));
    }

    fn with_tag_value(&mut self, value: &'a str) {
        self.tags
            .get_or_insert_with(|| Vec::new())
            .push((None, value));
    }

    fn build_base_metric(&self) -> String {
        // XXX: Wild guess, this /should/ be exactly what we need for the base
        // metric and even the tags that will be appended.
        let required = self.prefix.len() + self.key.len() + 10;

        let mut buf = String::with_capacity(required);
        let _ = write!(buf, "{}.{}:{}|{}", self.prefix, self.key, self.val, self.type_);
        buf
    }

    pub(crate) fn build(&self) -> T {
        let mut base = self.build_base_metric();
        if let Some(tags) = self.tags.as_ref() {
            push_datadog_tags(&mut base, tags);
        }

        T::from(base)
    }
}

#[derive(Debug, Clone)]
pub struct MetricBuilder<'m, 'c, T>
where
    T: Metric + From<String>,
{
    // TODO: Make this Option<Formatter> and Option<Error>?
    formatter: MetricFormatter<'m, T>,
    client: &'c StatsdClient,
}

impl<'m, 'c, T> MetricBuilder<'m, 'c, T>
where
    T: Metric + From<String>,
{
    pub(crate) fn new(formatter: MetricFormatter<'m, T>, client: &'c StatsdClient) -> Self {
        MetricBuilder {
            formatter: formatter,
            client: client,
        }
    }

    pub fn with_tag(&mut self, key: &'m str, value: &'m str) -> &mut Self {
        self.formatter.with_tag(key, value);
        self
    }

    pub fn with_tag_value(&mut self, value: &'m str) -> &mut Self {
        self.formatter.with_tag_value(value);
        self
    }

    pub fn send(&self) -> MetricResult<T> {
        let metric: T = self.formatter.build();
        self.client.send_metric(&metric)?;
        Ok(metric)
    }
}

fn push_datadog_tags(metric: &mut String, tags: &[(Option<&str>, &str)]) {
    // XXX: could return an Error if there's any empty strings
    let kv_size: usize = tags.iter()
        .map(|tag| {
            tag.0.map_or(0, |k| k.len() + 1) // +1 for : separator
             + tag.1.len()
        })
        .sum();

    // reserve enough space for prefix, tags/: separators and commas
    let prefix = "|#";
    let tags_size = prefix.len() + kv_size + tags.len() - 1;
    metric.reserve(tags_size);

    metric.push_str(prefix);
    for (i, &(key, value)) in tags.iter().enumerate() {
        if i > 0 {
            metric.push(',');
        }
        if let Some(key) = key {
            metric.push_str(key);
            metric.push(':');
        }
        metric.push_str(value);
    }
}

#[cfg(test)]
mod tests {
    use types::{Counter, Gauge, Histogram, Meter, Metric, Timer};
    use super::{push_datadog_tags, MetricFormatter};

    #[test]
    fn test_metric_formatter_counter_no_tags() {
        let fmt = MetricFormatter::counter("prefix", "some.key", 4);
        let counter: Counter = fmt.build();

        assert_eq!("prefix.some.key:4|c", counter.as_metric_str());
    }

    #[test]
    fn test_metric_formatter_counter_with_tags() {
        let mut fmt = MetricFormatter::counter("prefix", "some.key", 4);
        fmt.with_tag("host", "app03.example.com");
        fmt.with_tag("bucket", "2");
        fmt.with_tag_value("beta");

        let counter: Counter = fmt.build();

        assert_eq!(
            concat!(
                "prefix.some.key:4|c|#",
                "host:app03.example.com,",
                "bucket:2,",
                "beta",
            ),
            counter.as_metric_str()
        );
    }

    #[test]
    fn test_metric_formatter_timer_no_tags() {
        let fmt = MetricFormatter::timer("prefix", "some.method", 21);
        let timer: Timer = fmt.build();

        assert_eq!("prefix.some.method:21|ms", timer.as_metric_str());
    }

    #[test]
    fn test_metric_formatter_timer_with_tags() {
        let mut fmt = MetricFormatter::timer("prefix", "some.method", 21);
        fmt.with_tag("app", "metrics");
        fmt.with_tag_value("async");

        let timer: Timer = fmt.build();

        assert_eq!(
            "prefix.some.method:21|ms|#app:metrics,async",
            timer.as_metric_str()
        );
    }

    #[test]
    fn test_metric_formatter_gauge_no_tags() {
        let fmt = MetricFormatter::gauge("prefix", "num.failures", 7);
        let gauge: Gauge = fmt.build();

        assert_eq!("prefix.num.failures:7|g", gauge.as_metric_str());
    }

    #[test]
    fn test_metric_formatter_gauge_with_tags() {
        let mut fmt = MetricFormatter::gauge("prefix", "num.failures", 7);
        fmt.with_tag("window", "300");
        fmt.with_tag_value("best-effort");

        let gauge: Gauge = fmt.build();

        assert_eq!(
            "prefix.num.failures:7|g|#window:300,best-effort",
            gauge.as_metric_str()
        );
    }

    #[test]
    fn test_metric_formatter_meter_no_tags() {
        let fmt = MetricFormatter::meter("prefix", "user.logins", 3);
        let meter: Meter = fmt.build();

        assert_eq!("prefix.user.logins:3|m", meter.as_metric_str());
    }

    #[test]
    fn test_metric_formatter_meter_with_tags() {
        let mut fmt = MetricFormatter::meter("prefix", "user.logins", 3);
        fmt.with_tag("user-type", "verified");
        fmt.with_tag_value("bucket1");

        let meter: Meter = fmt.build();

        assert_eq!(
            "prefix.user.logins:3|m|#user-type:verified,bucket1",
            meter.as_metric_str()
        );
    }

    #[test]
    fn test_metric_formatter_histogram_no_tags() {
        let fmt = MetricFormatter::histogram("prefix", "num.results", 44);
        let histogram: Histogram = fmt.build();

        assert_eq!("prefix.num.results:44|h", histogram.as_metric_str());
    }

    #[test]
    fn test_metric_formatter_histogram_with_tags() {
        let mut fmt = MetricFormatter::histogram("prefix", "num.results", 44);
        fmt.with_tag("user-type", "authenticated");
        fmt.with_tag_value("source=search");

        let histogram: Histogram = fmt.build();

        assert_eq!(
            concat!(
                "prefix.num.results:44|h|#",
                "user-type:authenticated,",
                "source=search"
            ),
            histogram.as_metric_str()
        );
    }

    #[test]
    fn test_push_datadog_tags() {
        let metric_str = "some.counter:1|c";

        let mut m = metric_str.to_string();
        push_datadog_tags(&mut m, &vec![(Some("host"), "app01.example.com")]);
        assert_eq!(m, format!("{}|#host:app01.example.com", metric_str));

        let mut m = metric_str.to_string();
        push_datadog_tags(
            &mut m,
            &vec![
                (Some("host"), "app01.example.com"),
                (Some("bucket"), "A"),
                (None, "file-server"),
            ],
        );
        assert_eq!(
            m,
            format!(
                "{}|#host:app01.example.com,bucket:A,file-server",
                metric_str
            )
        );
    }
}

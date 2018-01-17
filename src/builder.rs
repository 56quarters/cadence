// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2018 Philip Jenvey <pjenvey@mozilla.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use client::StatsdClient;
use types::{Metric, MetricResult};

#[derive(Clone)]
pub struct MetricBuilder<'t, 'c, T>
where
    T: Metric,
{
    metric: T,
    tags: Option<Vec<(Option<&'t str>, &'t str)>>,
    client: &'c StatsdClient,
}

impl<'t, 'c, T> MetricBuilder<'t, 'c, T>
where
    T: Metric,
{
    pub fn new(metric: T, client: &'c StatsdClient) -> Self {
        MetricBuilder {
            metric: metric,
            tags: None,
            client: client,
        }
    }

    pub fn with_tag(&mut self, key: &'t str, value: &'t str) -> &mut Self {
        self.tags
            .get_or_insert_with(|| Vec::new())
            .push((Some(key), value));
        self
    }

    pub fn with_tag_value(&mut self, value: &'t str) -> &mut Self {
        self.tags
            .get_or_insert_with(|| Vec::new())
            .push((None, value));
        self
    }

    fn build(&self) -> String {
        let mut metric_string = self.metric.as_metric_str().to_string();
        if let Some(tags) = self.tags.as_ref() {
            push_datadog_tags(&mut metric_string, tags);
        }
        metric_string
    }

    pub fn send(&self) -> MetricResult<()> {
        let metric = MetricFromBuilder { repr: self.build() };
        self.client.send_metric(&metric)
    }
}

struct MetricFromBuilder {
    repr: String,
}

impl Metric for MetricFromBuilder {
    fn as_metric_str(&self) -> &str {
        &self.repr
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
    let prefix = "#|";
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
    use super::push_datadog_tags;

    #[test]
    fn test_push_datadog_tags() {
        let metric_str = "some.counter:1|c";

        let mut m = metric_str.to_string();
        push_datadog_tags(&mut m, &vec![(Some("host"), "app01.example.com")]);
        assert_eq!(m, format!("{}#|host:app01.example.com", metric_str));

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
                "{}#|host:app01.example.com,bucket:A,file-server",
                metric_str
            )
        );
    }
}

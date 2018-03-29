use client::{MetricClient, StatsdClient};
use types::MetricResult;
use builder::{MetricBuilder, MetricFormatter};
use datadog::types::{Event, Set, Distribution};
use datadog::builder::{EventBuilder, EventFormatter};
use super::types::{MAX_EVENT_LENGTH, max_event_exceeded_error};

/// Trait for recording Datadog set values.
///
/// Sets count the number of unique elements in a group. You can use them to,
/// for example, grouping the unique visitors to your site.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note that sets are a
/// [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) extension to
/// Statsd and may not be supported by your server.
pub trait Setted {
    /// Record a single set value with the given key
    fn set(&self, key: &str, value: i64) -> MetricResult<Set>;

    /// Record a single set value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    fn set_with_tags<'a>(&'a self, key: &'a str, value: i64) -> MetricBuilder<Set>;
}

/// Trait for recording Datadog distribution values.
///
/// Distribution values are positive values that can represent anything, whose
/// statistical distribution is calculated by the server. The values can be
/// timings, amount of some resource consumed, size of HTTP responses in
/// some application, etc. Histograms can be thought of as a more general
/// form of timers.
///
/// See the [Statsd spec](https://github.com/b/statsd_spec) for more
/// information.
///
/// Note: Distributions are a
/// [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) extension to
/// Statsd and are currently a beta feature of Datadog and not generally
/// available. Distributions must be specifically enabled for your
/// organization.
pub trait Distributed {
    /// Record a value to be tracked as a distribution to the statsd server.
    ///
    /// # Examples
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink};
    ///
    /// let client = StatsdClient::from_sink("myapp", NopMetricSink);
    /// client.distribution("some.distribution", 5);
    /// ```
    fn distribution(&self, key: &str, value: i64) -> MetricResult<Distribution>;

    /// Record a single distribution value with the given key and return a
    /// `MetricBuilder` that can be used to add tags to the metric.
    ///
    /// # Examples
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink};
    ///
    /// let client = StatsdClient::from_sink("myapp", NopMetricSink);
    /// client.distribution_with_tags("some.distribution", 5)
    ///     .with_tag("foo", "bar");
    /// ```
    fn distribution_with_tags<'a>(&'a self, key: &'a str, value: i64) -> MetricBuilder<Distribution>;
}

/// Trait for recording Datadog event values.
///
/// Events are values sent to the datadog event stream and can be queried via the
/// Datadog interface. You can tag them, set priority and even aggregate them with
/// other events.
///
/// They are composed by the following fields:
///
/// - Title — Event title (Required).
/// - Text — Event text/details (Required).
/// - Timestamp (Optional) — Add a timestamp to the event. Default is the current Unix
/// epoch timestamp.
/// - Hostname (Optional) - Add a hostname to the event. No default.
/// - Aggregation Key (Optional) — Add an aggregation key to group the event with others that have
/// the same key. No default.
/// - Priority (Optional) — Set to ‘normal’ or ‘low’. Default ‘normal’.
/// - Source Type Name (Optional) - Add a source type to the event. No default.
/// - Alert Type (Optional) — Set to ‘error’, ‘warning’, ‘info’ or ‘success’. Default ‘info’.
///
/// Note that events are a
/// [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) extension to
/// Statsd and may not be supported by your server.
pub trait Evented {
    /// Record a single event value with the given title and text
    /// using the following defaults for the rest of the fields:
    ///
    /// - Timestamp: The current Unix epoch timestamp.
    /// - Hostname: none.
    /// - Aggregation Key: none.
    /// - Priority: `EventPriority::Normal`.
    /// - Source Type Name: none.
    /// - Alert Type: `EventAlertType::Info`.
    ///
    /// # Examples
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink};
    ///
    /// let client = StatsdClient::from_sink("myapp", NopMetricSink);
    /// client.event("exception", "something bad happened");
    /// ```
    fn event(&self, title: &str, text: &str) -> MetricResult<Event>;

    /// Record a single event value with the given title and text
    /// returning a `MetricBuilder` that can be used to add tags
    /// and customize the event.
    ///
    /// # Examples
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, EventAlertType, EventPriority};
    ///
    /// let client = StatsdClient::from_sink("myapp", NopMetricSink);
    /// client.custom_event("exception", "server in flames!")
    ///     .with_timestamp(1523292353)
    ///     .with_hostname("example.com")
    ///     .with_aggregation_key("aggreg_key")
    ///     .with_priority(EventPriority::Low)
    ///     .with_src_type("src_type")
    ///     .with_alert_type(EventAlertType::Error)
    ///     .with_tag("foo", "bar")
    ///     .with_tag_value("quux")
    ///     .send();
    ///
    /// ```
    fn custom_event<'a>(&'a self, title: &'a str, text: &'a str) -> EventBuilder<Event>;
}

/// Trait that encompasses all other traits for sending metrics along with
/// Datadog extensions.
///
/// If you wish to use `StatsdClient` with a generic type or place a
/// `StatsdClient` instance behind a pointer (such as a `Box`) this will allow
/// you to reference all the implemented methods for recording metrics, while
/// using a single trait. An example of this is shown below.
///
/// ```
/// use cadence::{DatadogMetricClient, StatsdClient, NopMetricSink};
///
/// let client: Box<DatadogMetricClient> = Box::new(StatsdClient::from_sink(
///     "myapp", NopMetricSink));
///
/// client.count("some.counter", 1).unwrap();
/// client.time("some.timer", 42).unwrap();
/// client.gauge("some.gauge", 8).unwrap();
/// client.meter("some.meter", 13).unwrap();
/// client.histogram("some.histogram", 4).unwrap();
/// client.set("some.set", 5).unwrap();
/// client.distribution("some.distribution", 34).unwrap();
/// client.event("exception", "something bad happened").unwrap();
/// ```
pub trait DatadogMetricClient: MetricClient + Setted + Distributed + Evented {}

impl Setted for StatsdClient {
    fn set(&self, key: &str, value: i64) -> MetricResult<Set> {
        self.set_with_tags(key, value).try_send()
    }

    fn set_with_tags<'a>(&'a self, key: &'a str, value: i64) -> MetricBuilder<Set> {
        let fmt = MetricFormatter::set(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}

impl Distributed for StatsdClient {
    fn distribution(&self, key: &str, value: i64) -> MetricResult<Distribution> {
        self.distribution_with_tags(key, value).try_send()
    }

    fn distribution_with_tags<'a>(&'a self, key: &'a str, value: i64) -> MetricBuilder<Distribution> {
        let fmt = MetricFormatter::distribution(&self.prefix, key, value);
        MetricBuilder::new(fmt, self)
    }
}

impl Evented for StatsdClient {
    fn event(&self, title: &str, text: &str) -> MetricResult<Event> {
        self.custom_event(title, text).try_send()
    }

    fn custom_event<'a>(&'a self, title: &'a str, text: &'a str) -> EventBuilder<Event> {
        let fmt = EventFormatter::event(&self.prefix, title, text);

        let event_length = self.prefix.len() + title.len() + text.len();
        let builder: EventBuilder<Event> = if event_length > MAX_EVENT_LENGTH {
            EventBuilder::from_error(max_event_exceeded_error(), self)
        } else {
            EventBuilder::new(fmt, self)
        };

        builder
    }
}

impl DatadogMetricClient for StatsdClient {}

#[cfg(test)]
mod tests {
    use super::{Distributed, Setted, Evented, DatadogMetricClient, StatsdClient};
    use sinks::NopMetricSink;
    use types::Metric;

    /// Strips the timestamp which is by default dynamic and then uses regular assert_eq!
    macro_rules! assert_eq_ignore_timestamp {
        ($left:expr, $right:expr) => ({
            use regex::Regex;
            let re = Regex::new(r"\|d:\d+").unwrap();

            match (&$left, &$right) {
                (left_val, right_val) => {
                    let left_wo_timestamp = re.replace_all(*left_val, "");
                    let right_wo_timestamp = re.replace_all(*right_val, "");
                    assert_eq!(*left_wo_timestamp, right_wo_timestamp);
                }
            }
        });
    }

    #[test]
    fn test_statsd_client_set_no_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.set("some.set", 3);

        assert_eq!(
            "myapp.some.set:3|s",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_set_with_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .set_with_tags("some.set", 3)
            .with_tag("foo", "bar")
            .try_send();

        assert_eq!(
            "myapp.some.set:3|s|#foo:bar",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_distribution_no_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.distribution("some.distribution", 5);

        assert_eq!(
            "myapp.some.distribution:5|d",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_distribution_with_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .distribution_with_tags("some.distribution", 5)
            .with_tag("foo", "bar")
            .try_send();

        assert_eq!(
            "myapp.some.distribution:5|d|#foo:bar",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_event_no_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.event("exception", "something bad happened");

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_no_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.custom_event("exception", "something bad happened")
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_tags() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.custom_event("exception", "something bad happened")
            .with_tag("foo", "bar")
            .with_tag_value("baz")
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info|#foo:bar,baz",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_timestamp() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_timestamp(1523292353)
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info",
            res.unwrap().as_metric_str()
         );
    }

    #[test]
    fn test_statsd_client_custom_event_with_hostname() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_hostname("example.com")
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info|h:example.com",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_aggregation_key() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_aggregation_key("aggreg_key")
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info|k:aggreg_key",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_priority_low() {
        use datadog::builder::EventPriority;
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_priority(EventPriority::Low)
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:low|t:info",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_priority_normal() {
        use datadog::builder::EventPriority;
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_priority(EventPriority::Normal)
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_with_src_type_name() {
        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_src_type("src_type")
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info|s:src_type",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_alert_typei_info() {
        use datadog::builder::EventAlertType;

        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_alert_type(EventAlertType::Info)
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:info",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_alert_type_error() {
        use datadog::builder::EventAlertType;

        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_alert_type(EventAlertType::Error)
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:error",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_alert_type_warning() {
        use datadog::builder::EventAlertType;

        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_alert_type(EventAlertType::Warning)
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:warning",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_alert_type_warning_success() {
        use datadog::builder::EventAlertType;

        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad happened")
            .with_alert_type(EventAlertType::Success)
            .try_send();

        assert_eq_ignore_timestamp!(
            "_e{15,22}:myapp.exception|something bad happened|p:normal|t:success",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_custom_event_with_all() {
        use datadog::builder::EventAlertType;
        use datadog::builder::EventPriority;

        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client
            .custom_event("exception", "something bad!")
            .with_timestamp(1523292353)
            .with_hostname("example.com")
            .with_aggregation_key("aggreg_key")
            .with_priority(EventPriority::Low)
            .with_src_type("src_type")
            .with_alert_type(EventAlertType::Error)
            .with_tag("foo", "bar")
            .with_tag_value("quux")
            .try_send();

        assert_eq!(
            "_e{15,14}:myapp.exception|something bad!|d:1523292353|p:low|t:error|h:example.com|k:aggreg_key|s:src_type|#foo:bar,quux",
            res.unwrap().as_metric_str()
        );
    }

    #[test]
    fn test_statsd_client_event_max_length_exceeded() {
        use types::ErrorKind;
        use datadog::types::MAX_EVENT_LENGTH;

        let mut text = String::new();
        let letters = &mut ['L', 'O', 'L'].into_iter().cycle();
        for _ in 1..MAX_EVENT_LENGTH {
            text.push(*letters.next().unwrap());
        }

        let client = StatsdClient::from_sink("myapp", NopMetricSink);
        let res = client.event("exception", &text);

        assert!(res.is_err());
        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }

    // The following tests really just ensure that we've actually
    // implemented all the traits we're supposed to correctly. If
    // we hadn't, this wouldn't compile.

    #[test]
    fn test_statsd_client_as_setted() {
        let client: Box<Setted> = Box::new(StatsdClient::from_sink("myapp", NopMetricSink));

        client.set("some.set", 5).unwrap();
    }

    #[test]
    fn test_statsd_client_as_distributed() {
        let client: Box<Distributed> = Box::new(StatsdClient::from_sink("myapp", NopMetricSink));

        client.distribution("some.distribution", 20).unwrap();
    }

    #[test]
    fn test_statsd_client_as_evented() {
        let client: Box<Evented> = Box::new(StatsdClient::from_sink("myapp", NopMetricSink));

        client.event("exception", "something bad happened").unwrap();
    }

    #[test]
    fn test_statsd_client_as_datadog_metric_client() {
        let client: Box<DatadogMetricClient> = Box::new(StatsdClient::from_sink("myapp", NopMetricSink));

        client.count("some.counter", 3).unwrap();
        client.time("some.timer", 198).unwrap();
        client.gauge("some.gauge", 4).unwrap();
        client.meter("some.meter", 29).unwrap();
        client.histogram("some.histogram", 32).unwrap();
        client.set("some.set", 5).unwrap();
        client.distribution("some.distribution", 34).unwrap();
        client.event("exception", "something bad happened").unwrap();
    }
}

use std::fmt::{self, Write};
use std::marker::PhantomData;
use std::default::Default;
use std::time::{SystemTime, UNIX_EPOCH};

use types::{Metric, MetricError, MetricResult};
use builder::{datadog_tags_size_hint, write_datadog_tags};
use client::StatsdClient;
use super::types::{MAX_EVENT_LENGTH, max_event_exceeded_error};

// A UNIX timpestamp for the event
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
struct EventTimestamp(u64);

impl fmt::Display for EventTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for EventTimestamp {
    fn default() -> Self {
        EventTimestamp::now()
    }
}

impl EventTimestamp {
    fn now() -> Self {
        let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");

        EventTimestamp(since_epoch.as_secs())
    }

    fn from_secs(secs: u64) -> Self {
        EventTimestamp(secs)
    }
}

/// The priority of an event. Needed to use
/// [EventBuilder::with_priority](struct.EventBuilder.html#method.with_priority).
///
/// See [Datadog](https://docs.datadoghq.com/developers/dogstatsd/).
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum EventPriority {
    Low,
    Normal
}

impl fmt::Display for EventPriority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EventPriority::Low => "low".fmt(f),
            EventPriority::Normal => "normal".fmt(f)
        }
    }
}

impl Default for EventPriority {
    fn default() -> Self {
        EventPriority::Normal
    }
}

/// The the source type name of an event. Needed to use
///
/// See [Datadog](https://docs.datadoghq.com/developers/dogstatsd/).
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
struct EventSourceTypeName<'a>(&'a str);

impl<'a> fmt::Display for EventSourceTypeName<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The alert type of an event. Needed to use
/// [EventBuilder::with_priority](struct.EventBuilder.html#method.with_alert_type).
///
/// See [Datadog](https://docs.datadoghq.com/developers/dogstatsd/).
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum EventAlertType {
    Info,
    Error,
    Warning,
    Success
}

impl Default for EventAlertType {
    fn default() -> Self {
        EventAlertType::Info
    }
}

impl fmt::Display for EventAlertType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EventAlertType::Info => "info".fmt(f),
            EventAlertType::Error => "error".fmt(f),
            EventAlertType::Warning => "warning".fmt(f),
            EventAlertType::Success => "success".fmt(f)
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub(crate) struct EventFormatter<'a, T>
where
    T: Metric + From<String>,
{
    event: PhantomData<T>,
    prefix: &'a str,
    title: &'a str,
    text: &'a str,
    timestamp: EventTimestamp,
    hostname: Option<&'a str>,
    aggregation_key: Option<&'a str>,
    priority: EventPriority,
    src_type: Option<&'a str>,
    alert_type: EventAlertType,
    tags: Option<Vec<(Option<&'a str>, &'a str)>>,
    title_len: usize,
    text_len: usize,
}

impl<'a, T> EventFormatter<'a, T>
where
    T: Metric + From<String>,
{
    pub(crate) fn new(prefix: &'a str, title: &'a str, text:  &'a str) -> Self {
        let prefix_len = prefix.chars().count();
        let title_len = title.chars().count();
        let text_len = text.chars().count();

        EventFormatter {
            event: PhantomData,
            prefix: prefix,
            title: title,
            text: text,
            timestamp: Default::default(),
            hostname: None,
            aggregation_key: None,
            priority: Default::default(),
            src_type: None,
            alert_type: Default::default(),
            tags: None,
            title_len: title_len + prefix_len + 1, // 1 for '.'
            text_len: text_len,
        }
    }

    pub(crate) fn event(prefix: &'a str, title: &'a str, text:  &'a str) -> Self {
        Self::new(prefix, title, text)
    }

    fn with_timestamp(&mut self, timestamp: u64) {
        self.timestamp = EventTimestamp::from_secs(timestamp)
    }

    fn with_hostname(&mut self, hostname: &'a str) {
        self.hostname = Some(hostname)
    }

    fn with_aggregation_key(&mut self, key: &'a str) {
        self.aggregation_key = Some(key)
    }

    fn with_priority(&mut self, priority: EventPriority) {
        self.priority = priority
    }

    fn with_src_type_name(&mut self, src_type: &'a str) {
        self.src_type = Some(src_type)
    }

    fn with_alert_type(&mut self, alert_type: EventAlertType) {
        self.alert_type = alert_type
    }

    fn with_tag(&mut self, key: &'a str, value: &'a str) {
        self.tags
            .get_or_insert_with(Vec::new)
            .push((Some(key), value));
    }

    fn with_tag_value(&mut self, value: &'a str) {
        self.tags.get_or_insert_with(Vec::new).push((None, value));
    }

    fn write_base_metric(&self, out: &mut String) {
        let _ = write!(
            out,
            "_e{{{},{}}}:{}.{}|{}|d:{}|p:{}|t:{}",
            self.title_len,
            self.text_len,
            self.prefix,
            self.title,
            self.text,
            self.timestamp,
            self.priority,
            self.alert_type
        );

        if let Some(hostname) =  self.hostname.as_ref() {
            let _ = write!(out, "|h:{}", hostname);
        }

        if let Some(aggregation_key) =  self.aggregation_key.as_ref() {
            let _ = write!(out, "|k:{}", aggregation_key);
        }

        if let Some(src_type_name) =  self.src_type.as_ref() {
            let _ = write!(out, "|s:{}", src_type_name);
        }
    }

    fn write_tags(&self, out: &mut String) {
        if let Some(tags) = self.tags.as_ref() {
            write_datadog_tags(out, tags);
        }
    }

    // FIXME: This might be actually more expensive than resizing the string
    fn size_hint(&self) -> usize {
        let mut size = 2 +          // _e
            1 +                     // {
            4 +                     // title length 4 because of the max size len
            4 +                     // text length 4 because of the max size len
            2 +                     // }:
            self.prefix.len() + 1 + self.title.len() + 1 + // prefix + '.' + title + '|'
            self.text.len() + 1 +   // text_len and '|'
            13 +                    // 'd:' + bytes for the timestamp and '|'
            9 +                     // 'p:' bytes for max priority and '|'
            8;                      // 't:' bytes for max alert type and '|'

        if let Some(hostname) =  self.hostname.as_ref() {
            size += hostname.len() + 1 // hostname length + '|'
        }

        if let Some(aggregation_key) =  self.aggregation_key.as_ref() {
            size += aggregation_key.len() + 1 // aggregation_key + '|'
        }

        if let Some(src_type_name) =  self.src_type.as_ref() {
            size += src_type_name.len() + 1  // source type name + '|'
        }

        if let Some(tags) = self.tags.as_ref() {
            size + datadog_tags_size_hint(tags)
        } else {
            size
        }
    }

    pub(crate) fn build(&self) -> MetricResult<T> {
        let mut metric_string = String::with_capacity(self.size_hint());
        self.write_base_metric(&mut metric_string);
        self.write_tags(&mut metric_string);

        if metric_string.len() > MAX_EVENT_LENGTH {
            Err(max_event_exceeded_error())
        } else {
            Ok(T::from(metric_string))
        }
    }
}

/// Internal state of a `EventBuilder`
///
/// The builder can either be in the process of formatting a metric to send
/// via a client or it can be simply holding on to an error that it will return
/// to a caller when `.send()` is finally invoked.
#[derive(Debug)]
enum BuilderRepr<'m, 'c, T>
where
    T: Metric + From<String>,
{
    Success(EventFormatter<'m, T>, &'c StatsdClient),
    Error(MetricError, &'c StatsdClient),
}

/// Builder for customizing and adding tags to in-progress events.
///
/// This builder adds tags, key-value pairs or just values, as well as
/// custom values for the optional fields of an event previously constructed
/// by a call to `StatsdClient::custom_event`. The tags and customized 
/// values are added to events and sent via the client when
/// `EventBuilder::send()` is invoked. Any errors countered constructing,
/// validating, or sending the metrics will be propagated and returned when
/// the `.send()` method is finally invoked.
///
/// Datadog style tags are supported. For more information on the
/// exact format used, see the
/// [Datadog docs](https://docs.datadoghq.com/developers/dogstatsd/#datagram-format).
///
/// Adding tags to a metric via this builder will typically result in one or more
/// extra heap allocations.
///
/// Regarding the customizable values for the event fields you can send custom:
///
/// - Timestamp via `EventBuilder::with_timestamp()`.
/// - Hostname via `EventBuilder::with_hostname()`.
/// - Aggregation via `EventBuilder::with_aggregation_key()`.
/// - Priority via `EventBuilder::with_priority()` See `EventPriority` for possible
/// values.
/// - Source Type Name via `EventBuilder::with_src_type()`.
/// - Alert type `EventBuilder::with_alert_type()` See `EventAlertType` for possible
/// values.
///
/// Note that events are a [Datadog](https://docs.datadoghq.com/developers/dogstatsd/)
/// extension to Statsd and may not be supported by your server.
///
/// NOTE: The only way to instantiate an instance of this builder is via methods in
/// in the `StatsdClient` client.
///
/// # Example
///
/// An example of how the event builder is used with a `StatsdClient` instance
/// is given below.
///
/// ```
/// use cadence::prelude::*;
/// use cadence::{StatsdClient, NopMetricSink, Metric, EventPriority, EventAlertType};
///
/// let client = StatsdClient::from_sink("myapp", NopMetricSink);
/// let res = client.custom_event("exception", "something bad!")
///     .with_timestamp(1523292353)
///     .with_hostname("example.com")
///     .with_aggregation_key("aggreg_key")
///     .with_priority(EventPriority::Low)
///     .with_src_type("src_type")
///     .with_alert_type(EventAlertType::Error)
///     .try_send();
///
///     assert_eq!(
///         "_e{15,14}:myapp.exception|something bad!|d:1523292353|p:low|t:error|h:example.com|k:aggreg_key|s:src_type",
///         res.unwrap().as_metric_str()
///     );
/// ```
/// In this example, two key-value tags and one value tag are added to the
/// metric before it is finally sent to the Statsd server.
#[must_use = "Did you forget to call .send() after customizing the event?"]
#[derive(Debug)]
pub struct EventBuilder<'m, 'c, T>
where
    T: Metric + From<String>,
{
    repr: BuilderRepr<'m, 'c, T>,
}

impl<'m, 'c, T> EventBuilder<'m, 'c, T>
where
    T: Metric + From<String>,
{
    pub(crate) fn new(formatter: EventFormatter<'m, T>, client: &'c StatsdClient) -> Self {
        EventBuilder {
            repr: BuilderRepr::Success(formatter, client),
        }
    }

    pub(crate) fn from_error(err: MetricError, client: &'c StatsdClient) -> Self {
        EventBuilder {
            repr: BuilderRepr::Error(err, client),
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
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// let res = client.incr_with_tags("some.key")
    ///    .with_tag("user", "authenticated")
    ///    .try_send();
    ///
    /// assert_eq!(
    ///    "my.app.some.key:1|c|#user:authenticated",
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
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// let res = client.count_with_tags("some.key", 4)
    ///    .with_tag_value("beta-testing")
    ///    .try_send();
    ///
    /// assert_eq!(
    ///    "my.app.some.key:4|c|#beta-testing",
    ///    res.unwrap().as_metric_str()
    /// );
    /// ```
    pub fn with_tag_value(mut self, value: &'m str) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_tag_value(value);
        }

        self
    }

    /// Customize the timestamp of the event.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, Metric};
    ///
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// let res = client
    ///     .custom_event("exception", "something bad happened")
    ///     .with_timestamp(1523292353)
    ///     .try_send();
    /// ```
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_timestamp(timestamp);
        }

        self
    }

    /// Add a hostname to the event.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, Metric};
    ///
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// let res = client
    ///     .custom_event("exception", "something bad happened")
    ///     .with_hostname("example.com")
    ///     .try_send();
    /// ```
    pub fn with_hostname(mut self, hostname: &'m str) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_hostname(hostname);
        }

        self
    }

    /// Add an aggregation key to the event.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink};
    ///
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// let res = client
    ///     .custom_event("exception", "something bad happened")
    ///     .with_aggregation_key("aggregate_me")
    ///     .send();
    /// ```
    pub fn with_aggregation_key(mut self, key: &'m str) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_aggregation_key(key);
        }

        self
    }

   /// Customize the priority of the event.
   ///
   /// # Example
   ///
   /// ```
   /// use cadence::prelude::*;
   /// use cadence::{StatsdClient, NopMetricSink, EventPriority};
   ///
   /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
   /// let res = client
   ///     .custom_event("exception", "something bad happened")
   ///     .with_priority(EventPriority::Low)
   ///     .send();
   /// ```
   pub fn with_priority(mut self, priority: EventPriority) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_priority(priority);
        }

        self
    }

    /// Add a source type name to the event.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink};
    ///
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// let res = client
    ///     .custom_event("exception", "something bad happened")
    ///     .with_src_type("my_src_type_name")
    ///     .send();
    /// ```
    pub fn with_src_type(mut self, src_type_name: &'m str) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_src_type_name(src_type_name);
        }

        self
    }

    /// Customize the alert type of the event.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, EventAlertType};
    ///
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// let res = client
    ///     .custom_event("exception", "I am not ok with the events currently unfolding")
    ///     .with_alert_type(EventAlertType::Warning)
    ///     .send();
    /// ```
    pub fn with_alert_type(mut self, alert_type: EventAlertType) -> Self {
        if let BuilderRepr::Success(ref mut formatter, _) = self.repr {
            formatter.with_alert_type(alert_type);
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
                let metric: T = formatter.build()?;
                client.send_metric(&metric)?;
                Ok(metric)
            }
        }
    }

    /// Send an event using the client that created this builder.
    ///
    /// Note that the builder is consumed by this method and thus `.send()` can
    /// only be called a single time per builder.
    ///
    /// # Example
    ///
    /// ```
    /// use cadence::prelude::*;
    /// use cadence::{StatsdClient, NopMetricSink, Metric};
    ///
    /// let client = StatsdClient::from_sink("my.app", NopMetricSink);
    /// client.gauge_with_tags("some.key", 7)
    ///    .with_tag("test-segment", "12345")
    ///    .send();
    /// ```
    pub fn send(self) -> () {
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
    use super::*;
    use super::super::Event;
    // So, basically with these `size_hint` tests I'm trying to assert that at most we will do only
    // one allocation when creating the metric and that the amount of bytes allocated does not exceed
    // 16% from the real amount of bytes needed to store the metric string.
    // 16% Seems arbitrary but that was what I calculated from the curent `size_hint` implementation
    // without making it too expensive to call which would make it pointless.
    const SIZE_HINT_MAX_PERCENT: f32 = 0.16; // 16 %

    #[test]
    fn test_size_hint_default() {
        let ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_timestamp() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        // So by the time we really need a <u64>::max_value() we will not be around here
        // however the timestamp is u64 because of the Year 2038 problem.
        ef.with_timestamp(<u32>::max_value() as u64);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_hostname() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_hostname("example.com");

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_aggregation_key() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_aggregation_key("foo");

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_priority() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_priority(EventPriority::Low);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_normal() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_priority(EventPriority::Normal);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_src_type_name() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_src_type_name("bar");

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_alert_type() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_alert_type(EventAlertType::Error);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_alert_type_error() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_alert_type(EventAlertType::Error);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_alert_type_warning() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_alert_type(EventAlertType::Warning);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_alert_info() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_alert_type(EventAlertType::Info);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_alert_success() {
        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", "boom!");
        ef.with_alert_type(EventAlertType::Success);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_size_hint_with_all_long_text() {
        let really_long_text = r#"
            with an Apple Macintosh
            you can't run Radio Shack programs
            in its disc drive.

            nor can a Commodore 64
            drive read a file
            you have created on an
            IBM Personal Computer.

            both Kaypro and Osborne computers use
            the CP/M operating system
            but can't read each other's
            handwriting
            for they format (write
            on) discs in different
            ways.

            the Tandy 2000 runs MS-DOS but
            can't use most programs produced for
            the IBM Personal Computer
            unless certain
            bits and bytes are
            altered
            but the wind still blows over
            Savannah
            and in the Spring
            the turkey buzzard struts and
            flounces before his
            hens.
        "#;

        let mut ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", really_long_text);
        ef.with_timestamp(2147483647);
        ef.with_hostname("example.com");
        ef.with_aggregation_key("aggreg_key");
        ef.with_priority(EventPriority::Low);
        ef.with_src_type_name("src_type");
        ef.with_alert_type(EventAlertType::Error);

        let event: Event = ef.build().unwrap();

        let max_expected_gap = (event.as_metric_str().len() as f32 * SIZE_HINT_MAX_PERCENT) as usize;
        let size_hint_gap = ef.size_hint() - event.as_metric_str().len();
        assert!(size_hint_gap <= max_expected_gap);
    }

    #[test]
    fn test_formatter_max_event_length_error() {
        use types::ErrorKind;

        let mut text = String::new();
        let letters = &mut ['L', 'O', 'R'].into_iter().cycle();
        for _ in 0..MAX_EVENT_LENGTH {
            text.push(*letters.next().unwrap());
        }

        let ef: EventFormatter<Event> = EventFormatter::new("my.app", "exception", &text);
        let res = ef.build();

        assert!(res.is_err());
        assert_eq!(ErrorKind::InvalidInput, res.unwrap_err().kind());
    }
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2016 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::net::{ToSocketAddrs, UdpSocket};

use ::sinks::{MetricSink, UdpMetricSink};

use ::types::{MetricResult, Counter, Timer, Gauge, Meter, Metric};


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

    /// Decrement the counter by `1`
    fn decr(&self, key: &str) -> MetricResult<Counter>;

    /// Increment or decrement the counter by the given amount
    fn count(&self, key: &str, count: i64) -> MetricResult<Counter>;
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

    /// Record a meter value with the given key
    fn meter(&self, key: &str, value: u64) -> MetricResult<Meter>;
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
/// ```
pub trait MetricClient: Counted + Timed + Gauged + Metered {}


/// Client for Statsd that implements various traits to record metrics.
///
/// The client is the main entry point for users of this library. It supports
/// several traits for recording metrics of different types.
///
/// * `Counted` for emitting counters.
/// * `Timed` for emitting timings.
/// * `Gauged` for emitting gauge values.
/// * `Metered` for emitting meter values.
/// * `MetricClient` for a combination of all of the above.
///
/// For more information about the uses for each type of metric, see the
/// documentation for each mentioned trait.
///
/// The client uses some implementation of a `MetricSink` to emit the metrics.
///
/// In simple use cases when performance isn't critical, the `UdpMetricSink`
/// is likely the best choice since it is the simplest to use and understand.
///
/// When performance is more important, users will want to use the
/// `BufferedUdpMetricSink` in combination with the `AsyncMetricSink` for
/// maximum isolation between the sending metrics and your application as well
/// as minimum overhead when sending metrics.
#[derive(Debug, Clone)]
pub struct StatsdClient<T: MetricSink> {
    prefix: String,
    sink: T,
}


impl<T: MetricSink> StatsdClient<T> {
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
    pub fn from_sink(prefix: &str, sink: T) -> StatsdClient<T> {
        StatsdClient {
            prefix: trim_key(prefix).to_string(),
            sink: sink,
        }
    }

    /// Create a new client instance that will use the given prefix to send
    /// metrics to the given host over UDP using an appropriate sink. This is
    /// the construction method that most users of this library will use.
    ///
    /// The created UDP socket will be put into non-blocking mode.
    ///
    /// **Note** that you must include a type parameter when you call this
    /// method to help the compiler determine the type of `T` (the sink).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cadence::{StatsdClient, UdpMetricSink};
    ///
    /// let prefix = "my.stats";
    /// let host = ("metrics.example.com", 8125);
    ///
    /// // Note that we include a type parameter for the method call
    /// let client = StatsdClient::<UdpMetricSink>::from_udp_host(prefix, host);
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
    pub fn from_udp_host<A>(prefix: &str, host: A) -> MetricResult<StatsdClient<UdpMetricSink>>
        where A: ToSocketAddrs
    {
        let socket = try!(UdpSocket::bind("0.0.0.0:0"));
        try!(socket.set_nonblocking(true));
        let sink = try!(UdpMetricSink::from(host, socket));
        Ok(StatsdClient::from_sink(prefix, sink))
    }

    // Convert a metric to its Statsd string representation and then send
    // it as UTF-8 bytes to the metric sink. Convert any I/O errors from the
    // sink to MetricResults with the metric itself as a payload for success
    // responses.
    fn send_metric<M: Metric>(&self, metric: &M) -> MetricResult<()> {
        let metric_string = metric.as_metric_str();
        try!(self.sink.emit(metric_string));
        Ok(())
    }
}


impl<T: MetricSink> Counted for StatsdClient<T> {
    fn incr(&self, key: &str) -> MetricResult<Counter> {
        self.count(key, 1)
    }

    fn decr(&self, key: &str) -> MetricResult<Counter> {
        self.count(key, -1)
    }

    fn count(&self, key: &str, count: i64) -> MetricResult<Counter> {
        let counter = Counter::new(&self.prefix, key, count);
        try!(self.send_metric(&counter));
        Ok(counter)
    }
}


impl<T: MetricSink> Timed for StatsdClient<T> {
    fn time(&self, key: &str, time: u64) -> MetricResult<Timer> {
        let timer = Timer::new(&self.prefix, key, time);
        try!(self.send_metric(&timer));
        Ok(timer)
    }
}


impl<T: MetricSink> Gauged for StatsdClient<T> {
    fn gauge(&self, key: &str, value: u64) -> MetricResult<Gauge> {
        let gauge = Gauge::new(&self.prefix, key, value);
        try!(self.send_metric(&gauge));
        Ok(gauge)
    }
}


impl<T: MetricSink> Metered for StatsdClient<T> {
    fn mark(&self, key: &str) -> MetricResult<Meter> {
        self.meter(key, 1)
    }

    fn meter(&self, key: &str, value: u64) -> MetricResult<Meter> {
        let meter = Meter::new(&self.prefix, key, value);
        try!(self.send_metric(&meter));
        Ok(meter)
    }
}


impl<T: MetricSink> MetricClient for StatsdClient<T> {}


fn trim_key(val: &str) -> &str {
    if val.ends_with('.') {
        val.trim_right_matches('.')
    } else {
        val
    }
}


#[cfg(test)]
mod tests {
    use super::{trim_key, Counted, Timed, Gauged, Metered, MetricClient,
                StatsdClient};
    use ::sinks::NopMetricSink;

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
    fn test_statsd_client_as_metric_client() {
        let client: Box<MetricClient> = Box::new(StatsdClient::from_sink(
            "prefix", NopMetricSink));

        client.count("some.counter", 3).unwrap();
        client.time("some.timer", 198).unwrap();
        client.gauge("some.gauge", 4).unwrap();
        client.meter("some.meter", 29).unwrap();
    }
}

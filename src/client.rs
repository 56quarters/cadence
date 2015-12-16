use sinks::MetricSink;

use types::{
    MetricResult,
    Counter,
    Timer,
    Gauge,
    Meter,
    ToMetricString
};


/// Trait for incrementing and decrementing counters
///
/// Counters are simple values incremented or decremented by a client. The
/// rates at which these events occur or average values will be determined
/// by the server recieving them. Examples of counter uses include number
/// of logins to a system or requests recieved.
pub trait Counted {
    /// Increment the counter by `1`
    fn incr(&self, key: &str) -> MetricResult<()>;

    /// Decrement the counter by `1`
    fn decr(&self, key: &str) -> MetricResult<()>;

    /// Increment or decrement the counter by the given amount
    fn count(&self, key: &str, count: i64) -> MetricResult<()>;

    /// Increment or decrement the counter by the given amount
    /// at the specified sample rate (between `0.0` and `1.0`)
    fn sample(&self, key: &str, count: i64, sampling: f32) -> MetricResult<()>;
}


/// Trait for recording timings in milliseconds
///
/// Timings are a positive number of milliseconds between a start and end
/// time. Examples include time taken to render a web page or time taken
/// for a database call to return.
pub trait Timed {
    /// Record a timing in milliseconds under the given key
    fn time(&self, key: &str, time: u64) -> MetricResult<()>;
}


/// Trait for recording gauge values.
///
/// Gauge values are an instantaneous measurement of a value determined
/// by the client. They do not change unless changed by the client. Examples
/// include things like load average or how many connections are active.
pub trait Gauged {
    /// Record a gauge value under the given key
    fn gauge(&self, key: &str, value: u64) -> MetricResult<()>;
}


/// Trait for recording meter values.
///
/// Meter values measure the rate at which events occur. These rates are
/// determined by the server, the client simply indicates when they happen.
/// Meters can be thought of as increment-only counters. Examples include
/// things like number of requests handled or number of times something is
/// flushed to disk.
pub trait Metered {
    /// Record a single metered event under the given key
    fn mark(&self, key: &str) -> MetricResult<()>;

    /// Record a meter value under the given key
    fn meter(&self, key: &str, value: u64) -> MetricResult<()>;
}


///
pub struct StatsdClient<T: MetricSink> {
    key_gen: KeyGenerator,
    sink: T
}


impl<T: MetricSink> StatsdClient<T> {

    ///
    pub fn new(prefix: &str, sink: T) -> StatsdClient<T> {
        StatsdClient{
            key_gen: KeyGenerator::new(prefix),
            sink: sink
        }
    }

    //
    fn send_metric<M: ToMetricString>(&self, metric: &M) -> MetricResult<()> {
        let metric_string = metric.to_metric_string();
        let written = try!(self.sink.emit(&metric_string));
        debug!("Wrote {} ({} bytes)", metric_string, written);
        Ok(())
    }
}


impl<T: MetricSink> Counted for StatsdClient<T> {
    fn incr(&self, key: &str) -> MetricResult<()> {
        self.count(key, 1)
    }

    fn decr(&self, key: &str) -> MetricResult<()> {
        self.count(key, -1)
    }

    fn count(&self, key: &str, count: i64) -> MetricResult<()> {
        let counter = Counter::new(self.key_gen.make_key(key), count, None);
        self.send_metric(&counter)
    }

    fn sample(&self, key: &str, count: i64, sampling: f32) -> MetricResult<()> {
        let counter = Counter::new(self.key_gen.make_key(key), count, Some(sampling));
        self.send_metric(&counter)
    }
}


impl<T: MetricSink> Timed for StatsdClient<T> {
    fn time(&self, key: &str, time: u64) -> MetricResult<()> {
        let timer = Timer::new(self.key_gen.make_key(key), time);
        self.send_metric(&timer)
    }
}


impl<T: MetricSink> Gauged for StatsdClient<T> {
    fn gauge(&self, key: &str, value: u64) -> MetricResult<()> {
        let gauge = Gauge::new(self.key_gen.make_key(key), value);
        self.send_metric(&gauge)
    }
}


impl<T: MetricSink> Metered for StatsdClient<T> {

    fn mark(&self, key: &str) -> MetricResult<()> {
        self.meter(key, 1)
    }

    fn meter(&self, key: &str, value: u64) -> MetricResult<()> {
        let meter = Meter::new(self.key_gen.make_key(key), value);
        self.send_metric(&meter)
    }
}

///
struct KeyGenerator {
    prefix: String
}


impl KeyGenerator {
    ///
    fn new(prefix: &str) -> KeyGenerator {
        let trimmed = if prefix.ends_with('.') {
            prefix.trim_right_matches('.')
        } else {
            prefix
        };

        KeyGenerator{prefix: trimmed.to_string()}
    }

    ///
    fn make_key(&self, key: &str) -> String {
        format!("{}.{}", &self.prefix, key)
    }
}


#[cfg(test)]
mod tests {

    use super::KeyGenerator;
    
    #[test]
    fn test_key_generator_make_key_with_trailing_dot_prefix() {
        let key_gen = KeyGenerator::new("some.prefix.");
        assert_eq!("some.prefix.a.metric", key_gen.make_key("a.metric"));
    }

    #[test]
    fn test_key_generator_make_key_no_trailing_dot_prefix() {
        let key_gen = KeyGenerator::new("some.prefix");
        assert_eq!("some.prefix.a.metric", key_gen.make_key("a.metric"));
    }
}

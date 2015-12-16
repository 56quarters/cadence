//!
//!
//!

use sinks::MetricSink;

use types::{
    MetricResult,
    Counter,
    Timer,
    Gauge,
    Meter,
    ToMetricString
};


///
pub trait Counted {
    fn incr(&self, key: &str) -> MetricResult<()>;
    fn decr(&self, key: &str) -> MetricResult<()>;
    fn count(&self, key: &str, count: i64) -> MetricResult<()>;
    fn sample(&self, key: &str, count: i64, sampling: f32) -> MetricResult<()>;
}


///
pub trait Timed {
    fn time(&self, key: &str, time: u64) -> MetricResult<()>;
}


///
pub trait Gauged {
    fn gauge(&self, key: &str, value: u64) -> MetricResult<()>;
}


///
pub trait Metered {
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

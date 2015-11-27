//!
//!
//!

use client::net::MetricSink;
use client::types::{
    MetricResult,
    Counter,
    Timer,
    Gauge,
    ToMetricString
};



///
pub trait Counted {
    fn count(&self, key: &str, count: u64, sampling: Option<f32>) -> MetricResult<()>;
}


///
pub trait Timed {
    fn time(&self, key: &str, time: u64, sampling: Option<f32>) -> MetricResult<()>;
}


///
pub trait Gauged {
    fn gauge(&self, key: &str, value: i64) -> MetricResult<()>;
}


fn make_key(prefix: &str, key: &str) -> String {
    format!("{}.{}", prefix, key)
}


fn trim_prefix(prefix: &str) -> &str {
    if prefix.ends_with('.') {
        prefix.trim_right_matches('.')
    } else {
        prefix
    }
}


///
pub struct StatsdClient<T: MetricSink> {
    prefix: String,
    sink: Box<T>
}


impl<T: MetricSink> StatsdClient<T> {

    pub fn new(prefix: &str, sink: T) -> StatsdClient<T> {
        let trimmed = trim_prefix(prefix);
        StatsdClient{prefix: trimmed.to_string(), sink: Box::new(sink)}
    }
    
    fn send_metric<M: ToMetricString>(&self, metric: M) -> MetricResult<()> {
        let metric_string = metric.to_metric_string();
        let written = try!(self.sink.send(&metric_string));
        debug!("Wrote {} ({} bytes)", metric_string, written);
        Ok(())
    }
}


impl<T: MetricSink> Counted for StatsdClient<T> {
    fn count(&self, key: &str, count: u64, sampling: Option<f32>) -> MetricResult<()> {
        let key = make_key(&self.prefix, key);
        let counter = Counter::new(&key, count, sampling);
        self.send_metric(counter)
    }
}


impl<T: MetricSink> Timed for StatsdClient<T> {
    fn time(&self, key: &str, time: u64, sampling: Option<f32>) -> MetricResult<()> {
        let key = make_key(&self.prefix, key);
        let timer = Timer::new(&key, time, sampling);
        self.send_metric(timer)
    }
}


impl<T: MetricSink> Gauged for StatsdClient<T> {
    fn gauge(&self, key: &str, value: i64) -> MetricResult<()> {
        let key = make_key(&self.prefix, key);
        let gauge = Gauge::new(&key, value);
        self.send_metric(gauge)
    }
}


#[cfg(test)]
mod tests {

    use super::{make_key, trim_prefix};
    
    #[test]
    fn test_make_key() {
        let full_key = make_key("myapp.metrics", "foo.thing");
        assert_eq!("myapp.metrics.foo.thing".to_string(), full_key);
    }

    #[test]
    fn test_trim_prefix_with_trailing_dot() {
        let trimmed = trim_prefix("myapp.metrics.");
        assert_eq!("myapp.metrics".to_string(), trimmed);
    }

    #[test]
    fn test_trim_prefix_no_trailing_dot() {
        let trimmed = trim_prefix("myapp.metrics");
        assert_eq!("myapp.metrics".to_string(), trimmed);
    }
}

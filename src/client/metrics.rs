//!
//!
//!

use client::types::{
    MetricSink,
    MetricResult,
    Counter,
    Timer,
    Gauge,
    Counted,
    Timed,
    Gauged,
    ToMetricString
};


///
pub struct StatsdClient<T: MetricSink> {
    prefix: String,
    sink: Box<T>
}


impl<T: MetricSink> StatsdClient<T> {

    pub fn new(prefix: &str, sink: T) -> StatsdClient<T> {
        let trimmed = if prefix.ends_with('.') {
            prefix.trim_right_matches('.')
        } else {
            prefix
        };

        StatsdClient{prefix: trimmed.to_string(), sink: Box::new(sink)}
    }
    
    fn make_key(&self, key: &str) -> String {
        format!("{}.{}", &self.prefix, key)
    }
    
    fn send_metric<M: ToMetricString>(&self, metric: &M) -> MetricResult<()> {
        let metric_string = metric.to_metric_string();
        let written = try!(self.sink.send(&metric_string));
        debug!("Wrote {} ({} bytes)", metric_string, written);
        Ok(())
    }
}


impl<T: MetricSink> Counted for StatsdClient<T> {
    fn count(&self, key: &str, count: u64, sampling: Option<f32>) -> MetricResult<()> {
        let counter = Counter::new(self.make_key(key), count, sampling);
        self.send_metric(&counter)
    }
}


impl<T: MetricSink> Timed for StatsdClient<T> {
    fn time(&self, key: &str, time: u64, sampling: Option<f32>) -> MetricResult<()> {
        let timer = Timer::new(self.make_key(key), time, sampling);
        self.send_metric(&timer)
    }
}


impl<T: MetricSink> Gauged for StatsdClient<T> {
    fn gauge(&self, key: &str, value: i64) -> MetricResult<()> {
        let gauge = Gauge::new(self.make_key(key), value);
        self.send_metric(&gauge)
    }
}


#[cfg(test)]
mod tests {

    use super::StatsdClient;
    use client::sinks::NopMetricSink;
    
    #[test]
    fn test_statsd_client_make_key_with_trailing_dot_prefix() {
        let sink = NopMetricSink;
        let client = StatsdClient::new("some.prefix.", sink);
        assert_eq!("some.prefix.a.metric", client.make_key("a.metric"));
    }

    #[test]
    fn test_statsd_client_make_key_no_trailing_dot_prefix() {
        let sink = NopMetricSink;
        let client = StatsdClient::new("some.prefix", sink);
        assert_eq!("some.prefix.a.metric", client.make_key("a.metric"));
    }
}

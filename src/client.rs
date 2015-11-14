//
//
//
//
//

use std::io::Error;
use std::net::{ToSocketAddrs, UdpSocket};


pub const DEFAULT_PORT: u16 = 8125;


struct Counter<'a> {
    key: &'a str,
    count: u32,
    sampling: Option<f32>
}


struct Timer<'a> {
    key: &'a str,
    time: u32,
    unit: &'a str,
    sampling: Option<f32>
}


struct Gauge<'a> {
    key: &'a str,
    value: i32
}


trait ToMetricString {
    fn to_metric_string(&self) -> String;
}


impl<'a> ToMetricString for Counter<'a> {
    fn to_metric_string(&self) -> String {
        match self.sampling {
            Some(val) => format!("{}:{}|c|@{}", self.key, self.count, val),
            None => format!("{}:{}|c", self.key, self.count)
        }
    }
}


impl<'a> ToMetricString for Timer<'a> {
    fn to_metric_string(&self) -> String {
        match self.sampling {
            Some(val) => format!("{}:{}|{}|@{}", self.key, self.time, self.unit, val),
            None => format!("{}:{}|{}", self.key, self.time, self.unit)
        }
    }
}


impl<'a> ToMetricString for Gauge<'a> {
    fn to_metric_string(&self) -> String {
        format!("{}:{}|g", self.key, self.value)
    }
}


pub trait Counted {
    fn count(&self, key: &str, count: u32, sampling: Option<f32>) -> ();
}


pub trait Timed {
    fn time(&self, key: &str, time: u32, unit: &str, sampling: Option<f32>) -> ();
}


pub trait Gauged {
    fn gauge(&self, key: &str, value: i32) -> ();
}


pub trait ByteSink {
    fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize, Error>;
}


impl ByteSink for UdpSocket {
    fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize, Error> {
        self.send_to(buf, addr)
    }
}


pub struct StatsdClient<'a, T: ByteSink + 'a> {
    host: &'a str,
    port: u16,
    prefix: &'a str,
    sink: &'a T
}


impl<'a, T: ByteSink> StatsdClient<'a, T> {
    pub fn from_host(
        host: &'a str,
        port: u16,
        prefix: &'a str,
        sink: &'a T) -> StatsdClient<'a, T> {

        StatsdClient{
            host: host,
            port: port,
            prefix: prefix,
            sink: sink
        }
    }

    fn send_metric<B: ToMetricString>(&self, metric: B) -> () {
        let metric_string = metric.to_metric_string();
        let bytes = metric_string.as_bytes();
        let addr = (self.host, self.port);
        debug!("Sending to {}:{}", self.host, self.port);

        match self.sink.send_to(bytes, addr) {
            Ok(n) => debug!("Wrote {} bytes to socket", n),
            Err(err) => debug!("Got error writing to socket: {}", err)
        };
    }
}


fn make_key(prefix: &str, key: &str) -> String {
    let trimmed_prefix = if prefix.ends_with('.') {
        prefix.trim_right_matches('.')
    } else {
        prefix
    };

    format!("{}.{}", trimmed_prefix, key)
}


impl<'a, T: ByteSink> Counted for StatsdClient<'a, T> {
    fn count(&self, key: &str, count: u32, sampling: Option<f32>) -> () {
        let counter = Counter{
            key: &make_key(self.prefix, key), count: count, sampling: sampling};
        self.send_metric(counter);
    }
}


impl<'a, T: ByteSink> Timed for StatsdClient<'a, T> {
    fn time(&self, key: &str, time: u32, unit: &str, sampling: Option<f32>) -> () {
        let timer = Timer{
            key: &make_key(self.prefix, key), time: time, unit: unit, sampling: sampling};
        self.send_metric(timer);
    }
}


impl<'a, T: ByteSink> Gauged for StatsdClient<'a, T> {
    fn gauge(&self, key: &str, value: i32) -> () {
        let gauge = Gauge{key: &make_key(self.prefix, key), value: value};
        self.send_metric(gauge);
    }
}


#[cfg(test)]
mod tests {

    use super::{
        Counter,
        Timer,
        Gauge,
        ToMetricString,
        StatsdClient,
        Counted,
        Timed,
        Gauged,
        make_key
    };

    #[test]
    fn test_counter_to_metric_string_sampling() {
        let counter = Counter{key: "foo.bar", count: 4, sampling: Some(0.1)};
        assert_eq!("foo.bar:4|c|@0.1".to_string(), counter.to_metric_string());
    }

    #[test]
    fn test_counter_to_metric_string_no_sampling() {
        let counter = Counter{key: "foo.bar", count: 4, sampling: None};
        assert_eq!("foo.bar:4|c".to_string(), counter.to_metric_string());
    }

    #[test]
    fn test_timer_to_metric_string_sampling() {
        let timer = Timer{key: "foo.baz", time: 34, unit: "ms", sampling: Some(0.01)};
        assert_eq!("foo.baz:34|ms|@0.01".to_string(), timer.to_metric_string());
    }

    #[test]
    fn test_timer_to_metric_string_no_sampling() {
        let timer = Timer{key: "foo.baz", time: 34, unit: "ms", sampling: None};
        assert_eq!("foo.baz:34|ms".to_string(), timer.to_metric_string());
    }

    #[test]
    fn test_gauge_to_metric_string() {
        let gauge = Gauge{key: "foo.events", value: 2};
        assert_eq!("foo.events:2|g".to_string(), gauge.to_metric_string());
    }

    #[test]
    fn test_make_key_prefix_with_trailing_dot() {
        let full_key = make_key("myapp.metrics.", "foo.event");
        assert_eq!("myapp.metrics.foo.event".to_string(), full_key);
    }

    #[test]
    fn test_make_key_prefix_with_no_trailing_dot() {
        let full_key = make_key("myapp.metrics", "foo.thing");
        assert_eq!("myapp.metrics.foo.thing".to_string(), full_key);
    }
}

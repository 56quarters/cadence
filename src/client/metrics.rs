//!
//!
//!

use client::net::ByteSink;
use client::types::{Counter, Timer, Gauge, ToMetricString};

///
pub trait Counted {
    fn count(&self, key: &str, count: u32, sampling: Option<f32>) -> ();
}


///
pub trait Timed {
    fn time(&self, key: &str, time: u32, unit: &str, sampling: Option<f32>) -> ();
}


///
pub trait Gauged {
    fn gauge(&self, key: &str, value: i32) -> ();
}



///
pub struct StatsdClient<'a, T: ByteSink + 'a> {
    host: &'a str,
    port: u16,
    prefix: &'a str,
    sink: &'a T
}


impl<'a, T: ByteSink> StatsdClient<'a, T> {

    ///
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

    fn send_metric<M: ToMetricString>(&self, metric: M) -> () {
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
        let key = make_key(self.prefix, key);
        let counter = Counter::new(&key, count, sampling);
        self.send_metric(counter);
    }
}


impl<'a, T: ByteSink> Timed for StatsdClient<'a, T> {
    fn time(&self, key: &str, time: u32, unit: &str, sampling: Option<f32>) -> () {
        let key = make_key(self.prefix, key);
        let timer = Timer::new(&key, time, unit, sampling);
        self.send_metric(timer);
    }
}


impl<'a, T: ByteSink> Gauged for StatsdClient<'a, T> {
    fn gauge(&self, key: &str, value: i32) -> () {
        let key = make_key(self.prefix, key);
        let gauge = Gauge::new(&key, value);
        self.send_metric(gauge);
    }
}


#[cfg(test)]
mod tests {

    use super::make_key;
    
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

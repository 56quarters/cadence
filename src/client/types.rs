//!
//!
//!

use std::error;
use std::fmt;
use std::io;


///
pub struct Counter {
    key: String,
    count: u64,
    sampling: Option<f32>
}


///
pub struct Timer {
    key: String,
    time: u64,
    sampling: Option<f32>
}


///
pub struct Gauge {
    key: String,
    value: i64
}


impl Counter {
    pub fn new(key: &str, count: u64, sampling: Option<f32>) -> Counter {
        Counter{key: key.to_string(), count: count, sampling: sampling}
    }
}


impl Timer {
    pub fn new(key: &str, time: u64, sampling: Option<f32>) -> Timer {
        Timer{key: key.to_string(), time: time, sampling: sampling}
    }
}


impl Gauge {
    pub fn new (key: &str, value: i64) -> Gauge {
        Gauge{key: key.to_string(), value: value}
    }
}


///
pub trait ToMetricString {
    fn to_metric_string(&self) -> String;
}


impl ToMetricString for Counter {
    fn to_metric_string(&self) -> String {
        match self.sampling {
            Some(val) => format!("{}:{}|c|@{}", self.key, self.count, val),
            None => format!("{}:{}|c", self.key, self.count)
        }
    }
}


impl ToMetricString for Timer {
    fn to_metric_string(&self) -> String {
        match self.sampling {
            Some(val) => format!("{}:{}|ms|@{}", self.key, self.time, val),
            None => format!("{}:{}|ms", self.key, self.time)
        }
    }
}


impl ToMetricString for Gauge {
    fn to_metric_string(&self) -> String {
        format!("{}:{}|g", self.key, self.value)
    }
}

#[derive(Debug)]
pub enum MetricError {
    IoError(io::Error)
}


impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetricError::IoError(ref err) => write!(f, "IO error: {}", err)
        }
    }
}


impl error::Error for MetricError {
    fn description(&self) -> &str {
        match *self {
            MetricError::IoError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            MetricError::IoError(ref err) => Some(err)
        }
    }
}


impl From<io::Error> for MetricError {
    fn from(err: io::Error) -> MetricError {
        MetricError::IoError(err)
    }
}


pub type MetricResult<T> = Result<T, MetricError>;


#[cfg(test)]
mod tests {

    use super::{
        Counter,
        Timer,
        Gauge,
        ToMetricString
    };

    #[test]
    fn test_counter_to_metric_string_sampling() {
        let counter = Counter::new("foo.bar", 4, Some(0.1));
        assert_eq!("foo.bar:4|c|@0.1".to_string(), counter.to_metric_string());
    }

    #[test]
    fn test_counter_to_metric_string_no_sampling() {
        let counter = Counter::new("foo.bar", 4, None);
        assert_eq!("foo.bar:4|c".to_string(), counter.to_metric_string());
    }

    #[test]
    fn test_timer_to_metric_string_sampling() {
        let timer = Timer::new("foo.baz", 34, Some(0.01));
        assert_eq!("foo.baz:34|ms|@0.01".to_string(), timer.to_metric_string());
    }

    #[test]
    fn test_timer_to_metric_string_no_sampling() {
        let timer = Timer::new("foo.baz",34, None);
        assert_eq!("foo.baz:34|ms".to_string(), timer.to_metric_string());
    }

    #[test]
    fn test_gauge_to_metric_string() {
        let gauge = Gauge::new("foo.events", 2);
        assert_eq!("foo.events:2|g".to_string(), gauge.to_metric_string());
    }
}

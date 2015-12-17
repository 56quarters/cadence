use std::error;
use std::fmt;
use std::io;


///
pub struct Counter {
    key: String,
    count: i64,
    sampling: Option<f32>
}


impl Counter {
    ///
    pub fn new<S: Into<String>>(key: S, count: i64, sampling: Option<f32>) -> Counter {
        Counter{key: key.into(), count: count, sampling: sampling}
    }
}

///
pub struct Timer {
    key: String,
    time: u64
}


impl Timer {
    ///
    pub fn new<S: Into<String>>(key: S, time: u64) -> Timer {
        Timer{key: key.into(), time: time}
    }
}


///
pub struct Gauge {
    key: String,
    value: u64
}


impl Gauge {
    ///
    pub fn new<S: Into<String>>(key: S, value: u64) -> Gauge {
        Gauge{key: key.into(), value: value}
    }
}


///
pub struct Meter {
    key: String,
    value: u64
}


impl Meter {
    pub fn new<S: Into<String>>(key: S, value: u64) -> Meter {
        Meter{key: key.into(), value: value}
    }
}


///
pub trait ToMetricString {
    fn to_metric_string(&self) -> String;
}


impl ToMetricString for Counter {
    fn to_metric_string(&self) -> String {
        self.sampling.map_or_else(
            || format!("{}:{}|c", self.key, self.count),
            |rate| format!("{}:{}|c|@{}", self.key, self.count, rate))
    }
}


impl ToMetricString for Timer {
    fn to_metric_string(&self) -> String {
        format!("{}:{}|ms", self.key, self.time)
    }
}


impl ToMetricString for Gauge {
    fn to_metric_string(&self) -> String {
        format!("{}:{}|g", self.key, self.value)
    }
}


impl ToMetricString for Meter {
    fn to_metric_string(&self) -> String {
        format!("{}:{}|m", self.key, self.value)
    }
}



#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ErrorKind {
    InvalidInput,
    IoError,
}


#[derive(Debug)]
pub struct MetricError {
    repr: ErrorRepr
}


#[derive(Debug)]
enum ErrorRepr {
    WithDescription(ErrorKind, &'static str),
    IoError(io::Error)
}


impl MetricError {
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            ErrorRepr::IoError(_) => ErrorKind::IoError,
            ErrorRepr::WithDescription(kind, _) => kind
        }
    }
}


impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.fmt(f),
            ErrorRepr::WithDescription(_, desc) => desc.fmt(f)
        }
    }
}


impl error::Error for MetricError {
    fn description(&self) -> &str {
        match self.repr {
            ErrorRepr::IoError(ref err) => err.description(),
            ErrorRepr::WithDescription(_, desc) => desc
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self.repr {
            ErrorRepr::IoError(ref err) => Some(err),
            _ => None
        }
    }
}


impl From<io::Error> for MetricError {
    fn from(err: io::Error) -> MetricError {
        MetricError{repr: ErrorRepr::IoError(err)}
    }
}


impl From<(ErrorKind, &'static str)> for MetricError {
    fn from((kind, desc): (ErrorKind, &'static str)) -> MetricError {
        MetricError{repr: ErrorRepr::WithDescription(kind, desc)}
    }
}


pub type MetricResult<T> = Result<T, MetricError>;


#[cfg(test)]
mod tests {

    use super::{
        Counter,
        Timer,
        Gauge,
        Meter,
        ToMetricString
    };

    #[test]
    fn test_counter_to_metric_string_sampling() {
        let counter = Counter::new("test.counter", 4, Some(0.1));
        assert_eq!("test.counter:4|c|@0.1".to_string(), counter.to_metric_string());
    }

    #[test]
    fn test_counter_to_metric_string_no_sampling() {
        let counter = Counter::new("test.counter", 4, None);
        assert_eq!("test.counter:4|c".to_string(), counter.to_metric_string());
    }

    #[test]
    fn test_timer_to_metric_string() {
        let timer = Timer::new("test.timer", 34);
        assert_eq!("test.timer:34|ms".to_string(), timer.to_metric_string());
    }

    #[test]
    fn test_gauge_to_metric_string() {
        let gauge = Gauge::new("test.gauge", 2);
        assert_eq!("test.gauge:2|g".to_string(), gauge.to_metric_string());
    }

    #[test]
    fn test_meter_to_metric_string() {
        let meter = Meter::new("test.meter", 5);
        assert_eq!("test.meter:5|m".to_string(), meter.to_metric_string());
    }
}

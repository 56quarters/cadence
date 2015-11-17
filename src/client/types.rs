//!
//!
//!


///
pub struct Counter<'a> {
    key: &'a str,
    count: u32,
    sampling: Option<f32>
}


///
pub struct Timer<'a> {
    key: &'a str,
    time: u32,
    unit: &'a str,
    sampling: Option<f32>
}


///
pub struct Gauge<'a> {
    key: &'a str,
    value: i32
}


impl<'a> Counter<'a> {
    pub fn new(key: &'a str, count: u32, sampling: Option<f32>) -> Counter<'a> {
        Counter{key: key, count: count, sampling: sampling}
    }
}


impl<'a> Timer<'a> {
    pub fn new(key: &'a str, time: u32, unit: &'a str, sampling: Option<f32>) -> Timer<'a> {
        Timer{key: key, time: time, unit: unit, sampling: sampling}
    }
}


impl<'a> Gauge<'a> {
    pub fn new (key: &'a str, value: i32) -> Gauge<'a> {
        Gauge{key: key, value: value}
    }
}

///
pub trait ToMetricString {
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
}

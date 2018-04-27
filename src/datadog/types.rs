pub use types::Metric;
use builder::MetricFormatter;
use datadog::builder::EventFormatter;
use types::{MetricResult, MetricError, ErrorKind};

pub const MAX_EVENT_LENGTH: usize = 8 * 1024;

/// Sets count the number of unique elements in a group.
///
/// See the `Setted` trait for more information.
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Set {
    repr: String,
}

impl Set {
    pub fn new(prefix: &str, key: &str, value: i64) -> Set {
        MetricFormatter::set(prefix, key, value).build()
    }
}

impl From<String> for Set {
    fn from(s: String) -> Self {
        Set { repr: s }
    }
}

impl Metric for Set {
    fn as_metric_str(&self) -> &str {
        &self.repr
    }
}

/// Distributions track values to be computed as statistical
/// distributions.
/// 
/// See the `Distributed` trait for more information.
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Distribution {
    repr: String,
}

impl Distribution {
    pub fn new(prefix: &str, key: &str, value: i64) -> Set {
        MetricFormatter::distribution(prefix, key, value).build()
    }
}

impl From<String> for Distribution {
    fn from(s: String) -> Self {
        Distribution { repr: s }
    }
}

impl Metric for Distribution {
    fn as_metric_str(&self) -> &str {
        &self.repr
    }
}

/// Events are values sent to the datadog event stream and can be
/// via the Datadog interface.
///
/// See the `Evented` trait for more information.
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Event {
    repr: String,
}

impl Event {
    pub fn new(prefix: &str, title: &str, text: &str) -> MetricResult<Event> {
        EventFormatter::event(prefix, title, text).build()
    }
}

impl From<String> for Event {
    fn from(s: String) -> Self {
        Event { repr: s }
    }
}

impl Metric for Event {
    fn as_metric_str(&self) -> &str {
        &self.repr
    }
}

pub(crate) fn max_event_exceeded_error() -> MetricError {
    MetricError::from((ErrorKind::InvalidInput, "maximum event length exceeded"))
}

#[cfg(test)]
mod tests {
    use super::{Event, Metric, Set, Distribution};

    /// Strips the timestamp which is by default dynamic and then uses regular assert_eq!
    macro_rules! assert_eq_ignore_timestamp {
        ($left:expr, $right:expr) => ({
            use regex::Regex;
            let re = Regex::new(r"\|d:\d+").unwrap();

            match (&$left, &$right) {
                (left_val, right_val) => {
                    let left_wo_timestamp = re.replace_all(*left_val, "");
                    let right_wo_timestamp = re.replace_all(*right_val, "");
                    assert_eq!(*left_wo_timestamp, right_wo_timestamp);
                }
            }
        });
    }

    #[test]
    fn test_set_to_metric_string() {
        let set = Set::new("my.app", "test.set", 4);
        assert_eq!("my.app.test.set:4|s", set.as_metric_str());
    }

    #[test]
    fn test_distribution_to_metric_string() {
        let distribution = Distribution::new("my.app", "test.distribution", 5);
        assert_eq!("my.app.test.distribution:5|d", distribution.as_metric_str());
    }

    #[test]
    fn test_event_to_metric_string() {
        let event = Event::new("my.app", "boom!", "something bad happened").unwrap();

        assert_eq_ignore_timestamp!(
            "_e{12,22}:my.app.boom!|something bad happened|p:normal|t:info",
            event.as_metric_str()
        );
    }
}

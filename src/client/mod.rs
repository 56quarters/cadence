//!
//!
//!


pub const DEFAULT_PORT: u16 = 8125;

pub use self::metrics::{
    StatsdClient
};

pub use self::sinks::{
    UdpMetricSink,
    NopMetricSink
};

pub use self::types::{
    Counted,
    Timed,
    Gauged,
    MetricSink,
    MetricResult,
    MetricError
};

mod metrics;
mod sinks;
mod types;

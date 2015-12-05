//!
//!
//!


pub const DEFAULT_PORT: u16 = 8125;

pub use self::metrics::{
    Counted,
    Timed,
    Gauged,
    Metered,
    StatsdClient
};

pub use self::sinks::{
    MetricSink,
    UdpMetricSink,
    NopMetricSink
};

pub use self::types::{
    MetricResult,
    MetricError
};

mod metrics;
mod sinks;
mod types;

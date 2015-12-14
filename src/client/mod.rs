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
    NopMetricSink,
    UdpMetricSink
};


pub use self::types::{
    MetricResult,
    MetricError
};

mod metrics;
mod sinks;
mod types;

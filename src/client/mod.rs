//!
//!
//!


pub const DEFAULT_PORT: u16 = 8125;

pub use self::metrics::{
    Counted,
    Timed,
    Gauged,
    StatsdClient
};

pub use self::net::{
    MetricSink,
    UdpMetricSink,
    NopMetricSink
};

pub use self::types::{
    MetricResult,
    MetricError
};

mod metrics;
mod net;
mod types;

//!
//!
//!


pub const DEFAULT_PORT: u16 = 8125;

pub use client::metrics::{
    Counted,
    Timed,
    Gauged,
    StatsdClient
};

pub use client::net::{
    MetricSink,
    UdpMetricSink
};

pub use client::types::{
    MetricResult,
    MetricError
};

mod metrics;
mod net;
mod types;

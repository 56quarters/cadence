//!
//!
//!

#[macro_use]
extern crate log;

pub const DEFAULT_PORT: u16 = 8125;

pub use self::client::{
    Counted,
    Timed,
    Gauged,
    Metered,
    StatsdClient
};

pub use self::sinks::{
    MetricSink,
    ConsoleMetricSink,
    LoggingMetricSink,
    NopMetricSink,
    UdpMetricSink
};


pub use self::types::{
    MetricResult,
    MetricError,
    ErrorKind,
    Counter,
    Timer,
    Gauge,
    Meter
};

mod client;
mod sinks;
mod types;

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015 TSH Labs
//
// Available under the MIT license. See LICENSE for details.
//


//! # Cadence
//!
//! An extensible Statsd client for Rust!
//!
//! ## Features
//!
//!
//!
//! ## Install
//!
//!
//!
//! ## Usage
//!
//!
//!
//! ### Simple Use
//!
//!
//!
//! ### Counted, Timed, Gauged, and Metered Traits
//!
//!
//!
//! ### Custom Metric Sinks
//!
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

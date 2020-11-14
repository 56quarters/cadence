// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Export commonly used parts of Cadence for easy glob imports
//!
//! # Example
//!
//! ```
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, NopMetricSink};
//!
//! let client = StatsdClient::from_sink("some.prefix", NopMetricSink);
//!
//! client.count("some.counter", 1).unwrap();
//! client.time("some.timer", 23).unwrap();
//! client.gauge("some.gauge", 45).unwrap();
//! client.meter("some.meter", 67).unwrap();
//! client.histogram("some.histogram", 89).unwrap();
//! client.set("some.set", 123).unwrap();
//! ```

pub use crate::client::{Counted, CountedExt, Distributed, Gauged, Histogrammed, Metered, MetricClient, Setted, Timed};
pub use crate::compat::Compat;

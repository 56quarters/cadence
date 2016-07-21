// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2016 TSH Labs
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
//! ```no_run
//! use cadence::prelude::*;
//! use cadence::{DEFAULT_PORT, StatsdClient, UdpMetricSink};
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let client = StatsdClient::<UdpMetricSink>::from_udp_host("some.prefix", host);
//! ```

pub use ::client::{Counted, Timed, Gauged, Metered, MetricClient};

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2016 TSH Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


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

pub use client::{Counted, Timed, Gauged, Metered};

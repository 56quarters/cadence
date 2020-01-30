// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2018-2020 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Extension points for the Cadence library
//!
//! Libraries wishing to make use of Cadence for sending metrics to
//! a Statsd server but needing more control over how the metrics are
//! built and formatted can make use these extension points.

pub use crate::client::MetricBackend;

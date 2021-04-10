// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2018-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Advanced extension points for the Cadence library
//!
//! Most users of Cadence shouldn't need to make use of this module or
//! the included traits and types. However, users that need to extend the
//! library in unforeseen ways may find them useful.
//!
//! The `MetricBackend` trait, for example, can be used to implement a
//! client that sends a new non-standard type of metric using the same
//! backend that Cadence would use (via the `.send_metric()` method).
//!
//! The various `To*Value` traits are used as markers for types that are
//! valid for each type of metric. They also contain conversion logic for
//! the types in some cases (such as in the case of `Duration` objects).
//! These can be used to allow your own custom types to be converted to
//! metric values that Cadence understands.
//!
//! In summary, most users don't need to worry about these types but they
//! are available for advanced use cases and subject to the same guarantees
//! as the rest of the API (semantic versioning, etc.).

pub use crate::builder::MetricValue;
pub use crate::client::{
    MetricBackend, ToCounterValue, ToDistributionValue, ToGaugeValue, ToHistogramValue, ToMeterValue, ToSetValue,
    ToTimerValue,
};

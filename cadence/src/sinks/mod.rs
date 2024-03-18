// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod core;
mod queuing;
mod spy;
mod udp;

pub use crate::sinks::core::{MetricSink, NopMetricSink, SinkStats};
pub use crate::sinks::queuing::{QueuingMetricSink, QueuingMetricSinkBuilder};
pub use crate::sinks::spy::{BufferedSpyMetricSink, SpyMetricSink};
pub use crate::sinks::udp::{BufferedUdpMetricSink, UdpMetricSink};

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use crate::sinks::unix::{BufferedUnixMetricSink, UnixMetricSink};

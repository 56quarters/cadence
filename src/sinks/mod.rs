// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2019 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod queuing;
mod core;
mod udp;

pub use crate::sinks::queuing::QueuingMetricSink;
pub use crate::sinks::core::{MetricSink, NopMetricSink};
pub use crate::sinks::udp::{BufferedUdpMetricSink, UdpMetricSink};

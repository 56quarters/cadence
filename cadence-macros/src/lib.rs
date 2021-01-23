// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2020-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! An extensible Statsd client for Rust!
//!
//! Cadence is a fast and flexible way to emit Statsd metrics from your application.
//! The `cadence-macros` crate provides some wrappers to eliminate much of the boilerplate
//! that is often needed to emit metrics along with any tags that are associated with
//! them.
//!
//! ## Features
//!
//! * Support for emitting counters, timers, histograms, gauges, meters, and sets to
//!   Statsd over UDP (or optionally Unix sockets).
//! * Support for alternate backends via the `MetricSink` trait.
//! * Support for [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) style metrics tags.
//! * Macros to simplify common calls to emit metrics
//! * A simple yet flexible API for sending metrics.
//!
//! ## Install
//!
//! To make use of `cadence-macros` in your project, add it as a dependency in your `Cargo.toml` file.
//!
//! ```toml
//! [dependencies]
//! cadence-macros = "x.y.z"
//! ```
//!
//! ## Usage
//!
//! To make use of the macros in this crate, you'll need to set a global default Statsd client.
//! Configure a `cadence::StatsdClient` as usual and use the `set_global_default` function to
//! set it as the default. After that, you can make use of the macros in this crate.
//!
//! ```rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, QueuingMetricSink, BufferedUdpMetricSink, DEFAULT_PORT};
//! use cadence_macros::{statsd_count, statsd_time, statsd_gauge, statsd_meter, statsd_histogram, statsd_set};
//!
//! // Normal setup for a high-performance Cadence instance
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_nonblocking(true).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
//! let queuing_sink = QueuingMetricSink::from(udp_sink);
//! let client = StatsdClient::from_sink("my.prefix", queuing_sink);
//!
//! // Set the default client to use for macro calls
//! cadence_macros::set_global_default(client);
//!
//! // Macros!
//! statsd_count!("some.counter", 123);
//! statsd_count!("some.counter", 123, "tag" => "val");
//! statsd_count!("some.counter", 123, "tag" => "val", "another" => "thing");
//!
//! statsd_time!("some.timer", 123);
//! statsd_time!("some.timer", 123, "tag" => "val");
//! statsd_time!("some.timer", 123, "tag" => "val", "another" => "thing");
//!
//! statsd_gauge!("some.gauge", 123);
//! statsd_gauge!("some.gauge", 123, "tag" => "val");
//! statsd_gauge!("some.gauge", 123, "tag" => "val", "another" => "thing");
//!
//! statsd_meter!("some.meter", 123);
//! statsd_meter!("some.meter", 123, "tag" => "val");
//! statsd_meter!("some.meter", 123, "tag" => "val", "another" => "thing");
//!
//! statsd_histogram!("some.histogram", 123);
//! statsd_histogram!("some.histogram", 123, "tag" => "val");
//! statsd_histogram!("some.histogram", 123, "tag" => "val", "another" => "thing");
//!
//! statsd_set!("some.set", 123);
//! statsd_set!("some.set", 123, "tag" => "val");
//! statsd_set!("some.set", 123, "tag" => "val", "another" => "thing");
//! ```
//!
//! ## Limitations
//!
//! Some limitations with the current implemenation of Cadence macros are described below
//!
//! * Value tags are not supported. For example the following style of tag cannot be
//!   set when using macros: `client.count_with_tags("some.counter", 123).with_tag_value("beta").send()`
//! * Only a single type of value for each type of metric is supported. For example, only
//!   `u64` can be used with the `statsd_time!` macro, not a `std::time::Duration`. Only
//!   `u64` can be used with the `statsd_gauge!` macro, not a `f64`.
//!

pub use crate::state::{get_global_default, is_global_default_set, set_global_default, GlobalDefaultNotSet};

mod macros;
mod state;

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Datadog mextensions for Cadence.
//!
//! [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) defines few extensions to the
//! [Statsd](https://docs.datadoghq.com/developers/dogstatsd/) specification. Notably, two
//! additional types of metrics: Sets and the recently added Distributions as well as Events.
//!
//! This module implement these extensions in the case you are sending metrics to a fully 
//! compliant DogStatsD //! statsd server such as the 
//! [Datadog Agent](https://docs.datadoghq.com/agent/).
//!
//! These extensions are enabled when using this crate with the `datadog-extensions` feature
//! enabled.
//!
//! ## Features
//!
//! * Support for emitting sets and distributions to DogStatsD over UDP.
//! * Support for emitting events to DogStatsD over UDP.
//!
//! ## Usage
//!
//! Simple usage of Cadence Datadog extensions is shown below. In this example,
//! we just import the client, create an instance that will write to some 
//! imaginary metrics server, and send a few metrics.
//!
//! First, make sure you enable the extensions when pulling the dependency. Note the use of the
//! `datadog-extensions` feature.
//!
//! In your `Cargo.toml` file:
//! ``` toml
//! [dependencies.cadence]
//! version = "x.y.z"
//! features = ["datadog-extensions"]
//! ```
//!
//! ``` rust,no_run
//! // Import the client.
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
//!
//! // Create client that will write to the given host over UDP.
//! //
//! // Note that you'll probably want to actually handle any errors creating
//! // the client when you use it for real in your application. We're just
//! // using .unwrap() here since this is an example!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let client = StatsdClient::from_udp_host("my.metrics", host).unwrap();
//!
//! // Emit metrics!
//! client.set("mywebsite.users.uniques", 42);
//! client.distribution("mywebsite.page_render.time", 210);
//! client.event("exception", "something bad happened");
//! ```
pub mod client;
pub mod types;
pub mod builder;

#[cfg(feature = "datadog-extensions")]
pub use self::builder::{EventBuilder, EventPriority, EventAlertType};

#[cfg(feature = "datadog-extensions")]
pub use self::types::{Distribution, Event, Set};

#[cfg(feature = "datadog-extensions")]
pub use self::client::{Evented, Distributed, Setted, DatadogMetricClient};

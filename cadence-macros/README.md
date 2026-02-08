# cadence-macros

[![build status](https://circleci.com/gh/56quarters/cadence.svg?style=shield)](https://circleci.com/gh/56quarters/cadence)
[![docs.rs](https://docs.rs/cadence/badge.svg)](https://docs.rs/cadence-macros/)
[![crates.io](https://img.shields.io/crates/v/cadence-macros.svg)](https://crates.io/crates/cadence-macros/)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70+-lightgray.svg)](https://www.rust-lang.org)

[Cadence Documentation](https://docs.rs/cadence/)

[Macros Documentation](https://docs.rs/cadence-macros/)

An extensible Statsd client for Rust!

Cadence is a fast and flexible way to emit Statsd metrics from your application.
The `cadence-macros` crate provides some wrappers to eliminate much of the boilerplate
that is often needed to emit metrics along with any tags that are associated with
them.

## Features

* [Support](https://docs.rs/cadence/) for emitting counters, timers, histograms, distributions,
  gauges, meters, and sets to Statsd over UDP (or optionally Unix sockets).
* Support for alternate backends via the `MetricSink` trait.
* Support for [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) style metrics tags.
* [Macros](https://docs.rs/cadence-macros/) to simplify common calls to emit metrics
* A simple yet flexible API for sending metrics.

## Install

To make use of `cadence-macros` in your project, add it as a dependency in your `Cargo.toml` file.

```toml
[dependencies]
cadence-macros = "x.y.z"
```

## Usage

To make use of the macros in this crate, you'll need to set a global default Statsd client.
Configure a `cadence::StatsdClient` as usual and use the `set_global_default` function to
set it as the default. After that, you can make use of the macros in this crate.

```rust
use std::net::UdpSocket;
use std::time::Duration;
use cadence::prelude::*;
use cadence::{StatsdClient, QueuingMetricSink, BufferedUdpMetricSink, DEFAULT_PORT};
use cadence_macros::{statsd_count, statsd_time, statsd_gauge, statsd_meter, statsd_histogram, statsd_distribution, statsd_set};

// Normal setup for a high-performance Cadence instance
let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
socket.set_nonblocking(true).unwrap();

let host = ("metrics.example.com", DEFAULT_PORT);
let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
let queuing_sink = QueuingMetricSink::from(udp_sink);
let client = StatsdClient::from_sink("my.prefix", queuing_sink);

// Set the default client to use for macro calls
cadence_macros::set_global_default(client);

// Macros!
statsd_count!("some.counter", 123);
statsd_count!("some.counter", 123, "tag" => "val");
statsd_count!("some.counter", 123, "tag" => "val", "another" => "thing");

statsd_time!("some.timer", 123);
statsd_time!("some.timer", 123, "tag" => "val");
statsd_time!("some.timer", 123, "tag" => "val", "another" => "thing");
statsd_time!("some.timer", Duration::from_millis(123), "tag" => "val", "another" => "thing");

statsd_gauge!("some.gauge", 123);
statsd_gauge!("some.gauge", 123, "tag" => "val");
statsd_gauge!("some.gauge", 123, "tag" => "val", "another" => "thing");
statsd_gauge!("some.gauge", 123.123, "tag" => "val", "another" => "thing");

statsd_meter!("some.meter", 123);
statsd_meter!("some.meter", 123, "tag" => "val");
statsd_meter!("some.meter", 123, "tag" => "val", "another" => "thing");

statsd_histogram!("some.histogram", 123);
statsd_histogram!("some.histogram", 123, "tag" => "val");
statsd_histogram!("some.histogram", 123, "tag" => "val", "another" => "thing");
statsd_histogram!("some.histogram", Duration::from_nanos(123), "tag" => "val", "another" => "thing");
statsd_histogram!("some.histogram", 123.123, "tag" => "val", "another" => "thing");

statsd_distribution!("some.distribution", 123);
statsd_distribution!("some.distribution", 123, "tag" => "val");
statsd_distribution!("some.distribution", 123, "tag" => "val", "another" => "thing");
statsd_distribution!("some.distribution", 123.123, "tag" => "val", "another" => "thing");

statsd_set!("some.set", 123);
statsd_set!("some.set", 123, "tag" => "val");
statsd_set!("some.set", 123, "tag" => "val", "another" => "thing");
```

## Limitations

Some limitations with the current implemenation of Cadence macros are described below

* Value tags are not supported. For example the following style of tag cannot be
  set when using macros: `client.count_with_tags("some.counter", 123).with_tag_value("beta").send()`

## Other

For more information about Cadence, see the [README in the repository root](../README.md).

# Cadence

[![build status](https://circleci.com/gh/56quarters/cadence.svg?style=shield)](https://circleci.com/gh/56quarters/cadence)
[![docs.rs](https://docs.rs/cadence/badge.svg)](https://docs.rs/cadence/)
[![crates.io](https://img.shields.io/crates/v/cadence.svg)](https://crates.io/crates/cadence/)
[![Rust 1.36+](https://img.shields.io/badge/rust-1.36+-lightgray.svg)](https://www.rust-lang.org)

[Cadence Documentation](https://docs.rs/cadence/)

[Macros Documentation](https://docs.rs/cadence-macros/)

An extensible Statsd client for Rust!

Cadence is a fast and flexible way to emit Statsd metrics from your application.

## Features

* [Support](https://docs.rs/cadence/) for emitting counters, timers, histograms, distributions,
  gauges, meters, and sets to Statsd over UDP (or optionally Unix sockets).
* Support for alternate backends via the `MetricSink` trait.
* Support for [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) style metrics tags.
* [Macros](https://docs.rs/cadence-macros/) to simplify common calls to emit metrics
* A simple yet flexible API for sending metrics.

## Usage

An example of how to use Cadence for maximum performance is given below. For many more examples
and advanced use cases, see the [`cadence`](cadence) crate or the [documentation](https://docs.rs/cadence/).

```rust
use std::net::UdpSocket;
use cadence::prelude::*;
use cadence::{StatsdClient, QueuingMetricSink, BufferedUdpMetricSink, DEFAULT_PORT};

let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
socket.set_nonblocking(true).unwrap();

let host = ("metrics.example.com", DEFAULT_PORT);
let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
let queuing_sink = QueuingMetricSink::from(udp_sink);
let client = StatsdClient::from_sink("my.prefix", queuing_sink);

client.count("my.counter.thing", 29);
client.time("my.service.call", 214);
```

## Project layout

The [`cadence`](cadence) crate contains the Statsd client and primary API of Cadence. The
[`cadence-macros`](cadence-macros) crate contains optional  macros that can simplify use of
the Cadence API.

* [`cadence`](cadence): Statsd client and primary API
* [`cadence-macros`](cadence-macros): Optional convenience macros

## Documentation

The documentation is available at https://docs.rs/cadence/ or https://docs.rs/cadence-macros/

## Source

The source code is available on GitHub at https://github.com/56quarters/cadence

## Changes

Release notes for Cadence can be found in the [CHANGES.md](CHANGES.md) file.

## Development

Cadence uses Cargo for performing various development tasks.

To build Cadence:

```
$ cargo build
```

To run tests:

```
$ cargo test
```

or:

```
$ cargo test -- --ignored
```

To run benchmarks:

```
$ cargo bench
```

To build documentation:

```
$ cargo doc
```

## License

Licensed under either of
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

## Language Support

Cadence (latest master) supports building with a range of `1.36+` versions.

### Guaranteed to Build

The latest version of Cadence is tested against and will always build
correctly with

* The current `stable` version.
* The previous two stable versions, `stable - 1` and `stable - 2`.

### Best Effort Build

The latest version of Cadence is tested against and will *usually* build
correctly with

* The next two oldest stable versions, `stable - 3` and `stable - 4`.

Support for these versions may be dropped for a release in order to take
advantage of a feature available in newer versions of Rust.

### Known to Work

* Stable versions as far back as `1.36` are known to work with Cadence
  `0.21.0`. Building with this version (and any versions
  older than `stable - 4`) is not supported and may break at any time.

* Stable versions as far back as `1.34` are known to work with Cadence
  `0.20.0`. Building with this version (and any versions older than
  `stable - 4`) is not supported and may break at any time.

* Stable versions as far back as `1.32` are known to work with Cadence
  `0.19.0`. Building with this version (and any versions older than
  `stable - 4`) is not supported and may break at any time.

* Stable versions as far back as `1.31` are known to work with Cadence
  `0.18.0`. Building with this version (and any versions older than
  `stable - 4`) is not supported and may break at any time.

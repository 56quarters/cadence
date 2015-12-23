# Cadence

[![Build Status](https://travis-ci.org/tshlabs/cadence.svg?branch=master)](https://travis-ci.org/tshlabs/cadence)
[![crates.io](http://meritbadge.herokuapp.com/cadence)](https://crates.io/crates/cadence/)

An extensible Statsd client for Rust!

## Features

* Support for emitting counters, timers, gauges, and meters to Statsd over UDP.
* Support for alternate backends via the `MetricSink` trait.
* A simple yet flexible API for sending metrics.

## Install

To make use of Cadence in your project, add it as a dependency.

``` toml
[dependencies]
cadence = "x.y.z"
```

Then, link to it in your library or application.

``` rust
// bin.rs or lib.rs
extern crate cadence;

// rest of your library or application
```

## Usage

Typical usage of Cadence is shown below:

``` rust
// Import the client
use cadence::{
    StatsdClient,
    UdpMetricSink
};

// Create client that will write to the given host over UDP.
//
// Note that you'll probably want to actually handle any errors creating the client
// when you use it for real in your application. We're just using .unwrap() here
// since this is an example!
let host = ("metrics.example.com", 8125);
let client = StatsdClient::<UdpMetricSink>::from_host("my.metrics", host).unwrap();

// Emit metrics!
client.incr("some.counter");
client.time("some.methodCall", 42);
client.meter("some.value", 5);
```

## Documentation

Comming soon!

## Source

The source code is available on GitHub at https://github.com/tshlabs/cadence

## Changes

Release notes for Cadence can be found in the [CHANGES.md](CHANGES.md) file.


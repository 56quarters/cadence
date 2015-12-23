# Cadence

[![Build Status](https://travis-ci.org/tshlabs/cadence.svg?branch=master)](https://travis-ci.org/tshlabs/cadence)

An extensible Statsd client for Rust!

## Features

TBD

## Install

TBD

## Usage

Typical usage of Cadence is shown below:

``` rust
// Import the client
use cadence::{
    StatsdClient,
    UdpMetricsink
};

// Create client that will write to the given host over UDP.
//
// Note that you'll probably want to actually handle any errors creating the client
// when you use it for real in your application. Were just using .unwrap() here
// since this is an example!
let host = ("metrics.example.com", 8125);
let client = StatsdClient::<UdpMetricSink>::from_host("my.metrics", host).unwrap();

// Emit metrics!
client.incr("some.counter");
client.time("some.methodCall", 42);
client.meter("some.value", 5);
```

## Documentation

Documentation is available at http://tshlabs.github.io/cadence/

## Source

The source code is available on GitHub https://github.com/tshlabs/cadence

## Changes

Release notes for Cadence can be found in the [CHANGES.md](CHANGES.md) file.


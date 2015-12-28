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

Some examples of how to use Cadence are shown below.

### Simple Use

Simple usage of Cadence is shown below. In this example, we just import the client,
create an instance that will write to some imaginary metrics server, and send a few
metrics.

``` rust
// Import the client.
//
// Note that we're also importing each of the traits that the client uses to emit
// metircs (Counted, Timed, Gauged, and Metered).
use cadence::{
    Counted,
    Timed,
    Gauged,
    Metered,
    StatsdClient,
    UdpMetricSink,
    DEFAULT_PORT
};

// Create client that will write to the given host over UDP.
//
// Note that you'll probably want to actually handle any errors creating the client
// when you use it for real in your application. We're just using .unwrap() here
// since this is an example!
let host = ("metrics.example.com", DEFAULT_PORT);
let client = StatsdClient::<UdpMetricSink>::from_udp_host("my.metrics", host).unwrap();

// Emit metrics!
client.incr("some.counter");
client.time("some.methodCall", 42);
client.gauge("some.thing", 7);
client.meter("some.value", 5);
```

### Counted, Timed, Gauged, and Metered Traits

Each of the methods that the Cadence `StatsdClient` struct uses to send metrics are
implemented as a trait. If we want, we can just use the trait type to refer to the
client instance. This might be useful to you if you'd like to swap out the actual
Cadence client with a dummy version when you are unit testing your code.

``` rust
use cadence::{
    Counted,
    StatsdClient,
    UdpMetricSink,
    DEFAULT_PORT
};

pub struct User {
    id: u64,
    username: String,
    email: String
}

// Here's a simple DAO (Data Access Object) that doesn't do anything but
// uses a counter to keep track of the number of times the 'getUserById'
// method gets called.
pub struct MyUserDao<T: Counted> {
    counter: T
}

impl<T: Counted> MyUserDao<T> {
    // Create a new instance that will use the counter / client
    pub fn new(counter: T) -> MyUserDao<T> {
        MyUserDao{counter: counter}
    }

    /// Get a new user by their ID
    pub fn getUserById(&self, id: u64) -> Option<User> {
        self.counter.incr("getUserById");
        None
    }
}

fn main() {
    // Create a new Statsd client that writes to "metrics.example.com"
    let host = ("metrics.example.com", DEFAULT_PORT);
    let counter = StatsdClient::<UdpMetricSink>::from_udp_host(
        "counter.example", host).unwrap();

    // Create a new instance of the DAO that will use the client
    let dao = MyUserDao::new(counter);

    // Try to lookup a user by ID!
    match dao.getUserById(123) {
        Some(u) => println!("Found a user!"),
        None => println!("No user!")
    };
}

```

### Custom Metric Sinks

The Cadence `StatsdClient` uses implementations of the `MetricSink` trait to
send metrics to a metric server. Most users of the Candence library probably
want to use the `UdpMetricSink` implementation. This is the way people typically
interact with a Statsd server, sending packets over UDP.

However, maybe you'd like to do something custom: use a thread pool, send multiple
metrics at the same time, or something else. An example of creating a custom sink
is below.

``` rust
use std::io;
use cadence::{
    Counted,
    Timed,
    Gauged,
    Metered,
    StatsdClient,
    MetricSink,
    DEFAULT_PORT
};

pub struct MyMetricSink;

impl MetricSink for MyMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        // Your custom metric sink implementation goes here!
        Ok(0)
    }
}

fn main() {
    let sink = MyMetricSink;
    let client = StatsdClient::from_sink("my.prefix", sink);

    client.count("my.counter.thing", 42);
    client.time("my.method.time", 25);
    client.incr("some.other.counter");
}
```

## Documentation

The documentation is available at https://tshlabs.github.io/cadence/

## Source

The source code is available on GitHub at https://github.com/tshlabs/cadence

## Changes

Release notes for Cadence can be found in the [CHANGES.md](CHANGES.md) file.

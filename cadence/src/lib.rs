// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! An extensible Statsd client for Rust!
//!
//! Cadence is a fast and flexible way to emit Statsd metrics from your application.
//!
//! ## Features
//!
//! * Support for emitting counters, timers, histograms, distributions, gauges, meters, and sets to
//!   Statsd over UDP (or optionally Unix sockets).
//! * Support for alternate backends via the `MetricSink` trait.
//! * Support for [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) style metrics tags.
//! * Macros to simplify common calls to emit metrics
//! * A simple yet flexible API for sending metrics.
//!
//! ## Install
//!
//! To make use of `cadence` in your project, add it as a dependency in your `Cargo.toml` file.
//!
//! ```toml
//! [dependencies]
//! cadence = "x.y.z"
//! ```
//!
//! That's all you need!
//!
//! ## Usage
//!
//! Some examples of how to use Cadence are shown below. The examples start
//! simple and work up to how you should be using Cadence in a production
//! application.
//!
//! ### Simple Use
//!
//! Simple usage of Cadence is shown below. In this example, we just import
//! the client, create an instance that will write to some imaginary metrics
//! server, and send a few metrics.
//!
//! ```rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
//!
//! // Create client that will write to the given host over UDP.
//! //
//! // Note that you'll probably want to actually handle any errors creating
//! // the client when you use it for real in your application. We're just
//! // using .unwrap() here since this is an example!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! let sink = UdpMetricSink::from(host, socket).unwrap();
//! let client = StatsdClient::from_sink("my.metrics", sink);
//!
//! // Emit metrics!
//! client.incr("some.counter");
//! client.time("some.methodCall", 42);
//! client.gauge("some.thing", 7);
//! client.meter("some.value", 5);
//! ```
//!
//! ### Buffered UDP Sink
//!
//! While sending a metric over UDP is very fast, the overhead of frequent
//! network calls can start to add up. This is especially true if you are
//! writing a high performance application that emits a lot of metrics.
//!
//! To make sure that metrics aren't interfering with the performance of
//! your application, you may want to use a `MetricSink` implementation that
//! buffers multiple metrics before sending them in a single network
//! operation. For this, there's `BufferedUdpMetricSink`. An example of
//! using this sink is given below.
//!
//! ```rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, BufferedUdpMetricSink, DEFAULT_PORT};
//!
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_nonblocking(true).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
//! let client = StatsdClient::from_sink("my.prefix", sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! ```
//!
//! As you can see, using this buffered UDP sink is no more complicated
//! than using the regular, non-buffered, UDP sink.
//!
//! The only downside to this sink is that metrics aren't written to the
//! Statsd server until the buffer is full. If you have a busy application
//! that is constantly emitting metrics, this shouldn't be a problem.
//! However, if your application only occasionally emits metrics, this sink
//! might result in the metrics being delayed for a little while until the
//! buffer fills. In this case, it may make sense to use the `UdpMetricSink`
//! since it does not do any buffering.
//!
//! ### Queuing Asynchronous Metric Sink
//!
//! To make sure emitting metrics doesn't interfere with the performance
//! of your application (even though emitting metrics is generally quite
//! fast), it's probably a good idea to make sure metrics are emitted in
//! in a different thread than your application thread.
//!
//! To allow you to do this, there is `QueuingMetricSink`. This sink allows
//! you to wrap any other metric sink and send metrics to it via a queue,
//! as it emits metrics in another thread, asynchronously from the flow of
//! your application.
//!
//! The requirements for the wrapped metric sink are that it is thread
//! safe, meaning that it implements the `Send` and `Sync` traits. If
//! you're using the `QueuingMetricSink` with another sink from Cadence,
//! you don't need to worry: they are all thread safe.
//!
//! An example of using the `QueuingMetricSink` to wrap a buffered UDP
//! metric sink is given below. This is the preferred way to use Cadence
//! in production.
//!
//! ```rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, QueuingMetricSink, BufferedUdpMetricSink, DEFAULT_PORT};
//!
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_nonblocking(true).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
//! let queuing_sink = QueuingMetricSink::from(udp_sink);
//! let client = StatsdClient::from_sink("my.prefix", queuing_sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! ```
//!
//! In the example above, we use the default constructor for the queuing
//! sink which creates an **unbounded** queue, with no maximum size, to connect
//! the main thread where the client sends metrics to the background thread
//! in which the wrapped sink is running. If instead, you want to create a
//! **bounded** queue with a maximum size, you can use the `with_capacity`
//! constructor. An example of this is given below.
//!
//! ```rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, QueuingMetricSink, BufferedUdpMetricSink,
//!               DEFAULT_PORT};
//!
//! // Queue with a maximum capacity of 128K elements
//! const QUEUE_SIZE: usize = 128 * 1024;
//!
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_nonblocking(true).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
//! let queuing_sink = QueuingMetricSink::with_capacity(udp_sink, QUEUE_SIZE);
//! let client = StatsdClient::from_sink("my.prefix", queuing_sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! ```
//!
//! Using a `QueuingMetricSink` with a capacity set means that when the queue
//! is full, attempts to emit metrics via the `StatsdClient` will fail. While
//! this is bad, the alternative (if you instead used an unbounded queue) is
//! for unsent metrics to slowly use up more and more memory until your
//! application exhausts all memory.
//!
//! Using an **unbounded** queue means that the sending of metrics can absorb
//! slowdowns of sending metrics until your application runs out of memory.
//! Using a **bounded** queue puts a cap on the amount of memory that sending
//! metrics will use in your application. This is a tradeoff that users of
//! Cadence must decide for themselves.
//!
//! ### Use With Tags
//!
//! Adding tags to metrics is accomplished via the use of each of the `_with_tags`
//! methods that are part of the Cadence `StatsdClient` struct. An example of using
//! these methods is given below. Note that tags are an extension to the Statsd
//! protocol and so may not be supported by all servers.
//!
//! See the [Datadog docs](https://docs.datadoghq.com/developers/dogstatsd/) for
//! more information.
//!
//! ```rust,no_run
//! use cadence::prelude::*;
//! use cadence::{Metric, StatsdClient, NopMetricSink};
//!
//! let client = StatsdClient::from_sink("my.prefix", NopMetricSink);
//!
//! let res = client.count_with_tags("my.counter", 29)
//!     .with_tag("host", "web03.example.com")
//!     .with_tag_value("beta-test")
//!     .try_send();
//!
//! assert_eq!(
//!     concat!(
//!         "my.prefix.my.counter:29|c|#",
//!         "host:web03.example.com,",
//!         "beta-test"
//!     ),
//!     res.unwrap().as_metric_str()
//! );
//! ```
//!
//! ### Implemented Traits
//!
//! Each of the methods that the Cadence `StatsdClient` struct uses to send
//! metrics are implemented as a trait. There is also a trait that combines
//! all of these other traits. If we want, we can just use one of the trait
//! types to refer to the client instance. This might be useful to you if
//! you'd like to swap out the actual Cadence client with a dummy version
//! when you are unit testing your code or want to abstract away all the
//! implementation details of the client being used behind a trait and
//! pointer.
//!
//! Each of these traits are exported in the prelude module. They are also
//! available in the main module but aren't typically used like that.
//!
//! ```rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
//!
//! pub struct User {
//!     id: u64,
//!     username: String,
//!     email: String
//! }
//!
//!
//! // Here's a simple DAO (Data Access Object) that doesn't do anything but
//! // uses a metric client to keep track of the number of times the
//! // 'getUserById' method gets called.
//! pub struct MyUserDao {
//!     metrics: Box<dyn MetricClient>
//! }
//!
//!
//! impl MyUserDao {
//!     // Create a new instance that will use the StatsdClient
//!     pub fn new<T: MetricClient + 'static>(metrics: T) -> MyUserDao {
//!         MyUserDao { metrics: Box::new(metrics) }
//!     }
//!
//!     /// Get a new user by their ID
//!     pub fn get_user_by_id(&self, id: u64) -> Option<User> {
//!         self.metrics.incr("getUserById");
//!         None
//!     }
//! }
//!
//!
//! // Create a new Statsd client that writes to "metrics.example.com"
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! let sink = UdpMetricSink::from(host, socket).unwrap();
//! let metrics = StatsdClient::from_sink("counter.example", sink);
//!
//! // Create a new instance of the DAO that will use the client
//! let dao = MyUserDao::new(metrics);
//!
//! // Try to lookup a user by ID!
//! match dao.get_user_by_id(123) {
//!     Some(u) => println!("Found a user!"),
//!     None => println!("No user!")
//! };
//! ```
//!
//! ### Quiet Metric Sending and Error Handling
//!
//! When sending metrics sometimes you don't really care about the `Result` of
//! trying to send it or maybe you just don't want to deal with it inline with
//! the rest of your code. In order to handle this, Cadence allows you to set a
//! default error handler. This handler is invoked when there are errors sending
//! metrics so that the calling code doesn't have to deal with them.
//!
//! An example of configuring an error handler and an example of when it might
//! be invoked is given below.
//!
//! ```rust,no_run
//! use cadence::prelude::*;
//! use cadence::{MetricError, StatsdClient, NopMetricSink};
//!
//! fn my_error_handler(err: MetricError) {
//!     println!("Metric error! {}", err);
//! }
//!
//! let client = StatsdClient::builder("prefix", NopMetricSink)
//!     .with_error_handler(my_error_handler)
//!     .build();
//!
//! // When sending metrics via the `MetricBuilder` used for assembling tags,
//! // callers may opt into sending metrics quietly via the `.send()` method
//! // as opposed to the `.try_send()` method
//! client.count_with_tags("some.counter", 42)
//!     .with_tag("region", "us-east-2")
//!     .send();
//! ```
//!
//! ### Custom Metric Sinks
//!
//! The Cadence `StatsdClient` uses implementations of the `MetricSink`
//! trait to send metrics to a metric server. Most users of the Cadence
//! library probably want to use the `QueuingMetricSink` wrapping an instance
//! of the `BufferedMetricSink`.
//!
//! However, maybe you want to do something not covered by an existing sink.
//! An example of creating a custom sink is below.
//!
//! ```rust,no_run
//! use std::io;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, MetricSink, DEFAULT_PORT};
//!
//! pub struct MyMetricSink;
//!
//!
//! impl MetricSink for MyMetricSink {
//!     fn emit(&self, metric: &str) -> io::Result<usize> {
//!         // Your custom metric sink implementation goes here!
//!         Ok(0)
//!     }
//! }
//!
//!
//! let sink = MyMetricSink;
//! let client = StatsdClient::from_sink("my.prefix", sink);
//!
//! client.count("my.counter.thing", 42);
//! client.time("my.method.time", 25);
//! client.incr("some.other.counter");
//! ```
//!
//! ### Custom UDP Socket
//!
//! Most users of the Cadence `StatsdClient` will be using it to send metrics
//! over a UDP socket. If you need to customize the socket, for example you
//! want to use the socket in blocking mode but set a write timeout, you can
//! do that as demonstrated below.
//!
//! ```rust,no_run
//! use std::net::UdpSocket;
//! use std::time::Duration;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
//!
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_write_timeout(Some(Duration::from_millis(1))).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let sink = UdpMetricSink::from(host, socket).unwrap();
//! let client = StatsdClient::from_sink("my.prefix", sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! client.set("users.uniques", 42);
//! ```
//!
//! ### Unix Sockets
//!
//! Cadence also supports using Unix datagram sockets with the `UnixMetricSink`  or
//! `BufferedUnixMetricSink`. Unix sockets can be used for sending metrics to a server
//! or agent running on the same machine (physical machine, VM, containers in a pod)
//! as your application. Unix sockets are somewhat similar to UDP sockets with a few
//! important differences:
//!
//! * Sending metrics on a socket that doesn't exist or is not being listened to will
//!   result in an error.
//! * Metrics sent on a connected socket are guaranteed to be delievered (i.e. they are
//!   reliable as opposed to UDP sockets). However, it's still possible that the metrics
//!   won't be read by the server due to a variety of environment and server specific
//!   reasons.
//!
//! An example of using the sinks is given below.
//!
//! ```rust,no_run
//! use std::os::unix::net::UnixDatagram;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, BufferedUnixMetricSink};
//!
//! let socket = UnixDatagram::unbound().unwrap();
//! socket.set_nonblocking(true).unwrap();
//! let sink = BufferedUnixMetricSink::from("/run/statsd.sock", socket);
//! let client = StatsdClient::from_sink("my.prefix", sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! client.set("users.uniques", 42);
//! ```
//!
//! NOTE: This feature is only available on Unix platforms (Linux, BSD, MacOS).
//!

#![forbid(unsafe_code)]

pub const DEFAULT_PORT: u16 = 8125;

pub use self::builder::MetricBuilder;

pub use self::client::{
    Counted, Distributed, Gauged, Histogrammed, Metered, MetricClient, Setted, StatsdClient, StatsdClientBuilder, Timed,
};

pub use self::sinks::{
    BufferedSpyMetricSink, BufferedUdpMetricSink, MetricSink, NopMetricSink, QueuingMetricSink, SpyMetricSink,
    UdpMetricSink,
};

pub use self::types::{Counter, Distribution, ErrorKind, Gauge, Histogram, Meter, Metric, MetricError, MetricResult, Set, Timer};

mod builder;
mod client;
pub mod ext;
mod io;
pub mod prelude;
mod sinks;
mod types;

// Utilities for running integration tests with Unix datagram sockets.
#[cfg(unix)]
#[doc(hidden)]
pub mod test;

// Sinks for sending metrics over Unix datagram sockets
#[cfg(unix)]
pub use crate::sinks::{BufferedUnixMetricSink, UnixMetricSink};

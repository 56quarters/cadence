# Changelog

## [v0.16.0](https://github.com/tshlabs/cadence/tree/0.16.0) - 2018-12-07
* **Breaking change** - Require that all sinks and error handlers used with
  `StatsdClient` are panic safe, that is, they implement `RefUnwindSafe` per
  [#77](https://github.com/tshlabs/cadence/issues/77). Note that all sinks
  included with Cadence are panic safe so this shouldn't be much of a change
  for many users. See also [Rust #54768](https://github.com/rust-lang/rust/issues/54768)
  for more information about the reasoning for the change.

## [v0.15.1](https://github.com/tshlabs/cadence/tree/0.15.1) - 2018-07-19
* Update Cadence crate to forbid any uses of `unsafe {}` code.
* Minor documentation improvements.

## [v0.15.0](https://github.com/tshlabs/cadence/tree/0.15.0) - 2018-07-12
* **Breaking change** - Add support for `Set` metric types. Sets can be used
  to count the number of unique occurences of an event. Per
  [#62](https://github.com/tshlabs/cadence/pull/72).
* Updated dependency on `crossbeam` to the latest version (0.3.2).

## [v0.14.0](https://github.com/tshlabs/cadence/tree/0.14.0) - 2018-04-11
* **Breaking change** - Rename the `MetricBuilder::send()` method to
  `MetricBuilder::try_send()` and create a new `.send()` method that discards
  successful results and invokes a custom handler for error results. Handlers
  can be set by using a builder via the `StatsdClient::builder()` method.
  Per [#65](https://github.com/tshlabs/cadence/issues/65).

## [v0.13.2](https://github.com/tshlabs/cadence/tree/0.13.2) - 2018-03-13
* Warn when `MetricBuilder` instances aren't used when adding tags to metrics
  per [#63](https://github.com/tshlabs/cadence/issues/63).

## [v0.13.1](https://github.com/tshlabs/cadence/tree/0.13.1) - 2018-02-07
* Minor documentation improvements.

## [v0.13.0](https://github.com/tshlabs/cadence/tree/0.13.0) - 2018-02-06
* **Breaking change** - Added `_with_tags` method variants to all traits for
  emitting metrics (`Counted`, `Timed`, `Gauged`, `Metered`, `Histogrammed`)
  per [#41](https://github.com/tshlabs/cadence/issues/41). These methods will
  return a `MetricBuilder` instance that can be used to add
  [Datadog](https://docs.datadoghq.com/developers/dogstatsd/) style tags to
  metrics. Tags are an extension so they may not be supported by all Statsd
  servers.
* The `Metric` trait (which is used by each type of metric object for returning
  a `&str` representation of itself) is now part of the public API.

## [v0.12.2](https://github.com/tshlabs/cadence/tree/0.12.2) - 2017-11-29
* Fix off-by-one bug in underlying functionality for `BufferedUdpSink`
  that would have caused extra writes to the UDP socket per
  [#59](https://github.com/tshlabs/cadence/issues/59).

## [v0.12.1](https://github.com/tshlabs/cadence/tree/0.12.1) - 2017-09-21
* Minor documentation improvements and code cleanup.

## [v0.12.0](https://github.com/tshlabs/cadence/tree/0.12.0) - 2017-02-09
* Add new `time_duration` method to `Timed` trait per
  [#48](https://github.com/tshlabs/cadence/issues/48). This allows users to record
  timings using the `Duration` struct from the standard library.
* Add examples of Cadence usage per [#36](https://github.com/tshlabs/cadence/issues/36).

## [v0.11.0](https://github.com/tshlabs/cadence/tree/0.11.0) - 2017-01-18
* **Breaking change** - Remove deprecated `AsyncMetricSink` per
  [#47](https://github.com/tshlabs/cadence/issues/47). Users are encouraged to
  switch to `QueuingMetricSink` instead. `QueuingMetricSink` has similar performance,
  emits metrics asynchronously in another thread, and has a more ergonomic signature
  (not requiring a generic parameter for the wrapped sink).
* **Breaking change** - Remove the generic parameter `T` from the `StatsdClient` per
  [#45](https://github.com/tshlabs/cadence/issues/45). Instead of requiring all users
  of the client to care about the `MetricSink` implementation, put it behind an `Arc`
  pointer in the client and remove the type `T` from the signature. This makes the
  client easier to use and share between threads.
* Remove use of `Arc` inside various sinks per [#35](https://github.com/tshlabs/cadence/issues/35).

## [v0.10.0](https://github.com/tshlabs/cadence/tree/0.10.0) - 2017-01-08
* **Breaking change** - Remove deprecated `ConsoleMetricSink` and `LoggingMetricSink`
  per [#46](https://github.com/tshlabs/cadence/issues/46). Users wishing to still use
  these sinks are encouraged to
  [copy the code](https://github.com/tshlabs/cadence/blob/0.9.1/src/sinks/mod.rs)
  into their own projects or use Cadence version 0.9.1 until they migrate away from them.
* Deprecate `AsyncMetricSink` per [#34](https://github.com/tshlabs/cadence/issues/34).
  Anyone still using `AsyncMetricSink` is encouraged to switch to `QueuingMetricSink`
  instead. Performance should be comparable but `QueuingMetricSink` can be shared
  between threads without requiring a `.clone()`.

## [v0.9.1](https://github.com/tshlabs/cadence/tree/0.9.1) - 2017-01-01
* Change deprecation version of `LoggingMetricSink` and `ConsoleMetricSink` to 0.10.0.

## [v0.9.0](https://github.com/tshlabs/cadence/tree/0.9.0) - 2017-01-01
* Implement `QueuingMetricSink` utilizing a lock-free queue from the Crossbeam
  library per [#30](https://github.com/tshlabs/cadence/issues/30).
* Add new metric type, histograms, per [#40](https://github.com/tshlabs/cadence/issues/40).
* Deprecate `LoggingMetricSink` per [#32](https://github.com/tshlabs/cadence/issues/32).
* Deprecate `ConsoleMetricSink` per [#33](https://github.com/tshlabs/cadence/issues/33).

## [v0.8.2](https://github.com/tshlabs/cadence/tree/0.8.2) - 2016-12-12
* Internal code cleanup per [#29](https://github.com/tshlabs/cadence/issues/29).

## [v0.8.1](https://github.com/tshlabs/cadence/tree/0.8.1) - 2016-10-11
* Minor documentation fixes.

## [v0.8.0](https://github.com/tshlabs/cadence/tree/0.8.0) - 2016-08-27
* Add new `BufferedUdpMetricSink` implementation of a `MetricSink` that
  buffers multiple metrics before sending then in a single network operation
  per [#18](https://github.com/tshlabs/cadence/issues/18).
* Add new `AsyncMetricSink` implementation of a `MetricSink` that wraps
  another sink and sends metrics asynchronously using a thread pool per
  [#23](https://github.com/tshlabs/cadence/issues/23).
* Implement `Clone` trait for all builtin sinks for easier use with multiple
  threads, specifically the `AsyncMetricSink` per
  [#24](https://github.com/tshlabs/cadence/issues/24).

## [v0.7.0](https://github.com/tshlabs/cadence/tree/0.7.0) - 2016-07-27
* Add new `MetricClient` trait implemented by `StatsdClient` that encompasses
  all of the other traits for emitting metrics (`Counted`, `Timed`, `Gauged`,
  and `Metered`) so that users can refer to a single type when used with
  generics or behind a pointer per [#20](https://github.com/tshlabs/cadence/issues/20).

## [v0.6.0](https://github.com/tshlabs/cadence/tree/0.6.0) - 2016-07-20
* Change Cadence to be dual licensed under Apache and MIT licenses per
  [#12](https://github.com/tshlabs/cadence/issues/12).
* Improve documentation around `MetricSink` trait per
  [#13](https://github.com/tshlabs/cadence/issues/13).
* **Behavior change** - Change UDP sockets created by
  `StatsdClient::from_udp_host` to be created in non-blocking mode by default
  per [#14](https://github.com/tshlabs/cadence/issues/14). While this does
  change previous behavior, users of the library shouldn't notice much of
  a change. In instances where the caller would have blocked before, they
  will get a `MetricError` wrapping an `io::Error` (with an `ErrorKind` of
  `WouldBlock`). Users wishing to restore the old behavior can do so by
  creating a custom instance of `UdpMetricSink`. Thanks to the
  [Tikv](https://github.com/pingcap/tikv) team for the inspiration.

## [v0.5.2](https://github.com/tshlabs/cadence/tree/0.5.2) - 2016-07-02
* Increase test coverage per [#10](https://github.com/tshlabs/cadence/issues/10).
* Add documentation for setting up a UDP socket in non-blocking mode per
  [#8](https://github.com/tshlabs/cadence/issues/8).

## [v0.5.1](https://github.com/tshlabs/cadence/tree/0.5.1) - 2016-06-07
* Remove `debug!` call in internal StatsdClient call to cut down on log
  noise per [#7](https://github.com/tshlabs/cadence/pull/7).

## [v0.5.0](https://github.com/tshlabs/cadence/tree/0.5.0) - 2016-03-10
* **Breaking change** - Rename the constructor of `UdpMetricSink` from `new`
  to `from` to better match Rust naming conventions for conversion constructors.

## [v0.4.0](https://github.com/tshlabs/cadence/tree/0.4.0) - 2016-02-18
* Change name of method for getting metric `&str` representation. The old name
  implied that the instance was consumed which it was not.
* Create `cadence::prelude` module for easy import of `Counted`, `Timed`,
  `Gauged`, and `Metered` traits via a glob import. Fixes
  [#4](https://github.com/tshlabs/cadence/issues/4).

## [v0.3.0](https://github.com/tshlabs/cadence/tree/0.3.0) - 2016-02-07
* Change `LoggingMetricSink` log target to `cadence::metrics`.
* Minor documentation improvements. Fixes [#1](https://github.com/tshlabs/cadence/issues/1).
* Add benchmarks to test suite.
* Reduce heap allocations when emitting metrics. Fixes
  [#3](https://github.com/tshlabs/cadence/issues/3).

## [v0.2.1](https://github.com/tshlabs/cadence/tree/0.2.1) - 2015-12-27
* Change Cadence from MIT license to Apache-2.0 for better compatibility with
  BSD and GPLv3 licensed code.

## [v0.2.0](https://github.com/tshlabs/cadence/tree/0.2.0) - 2015-12-26
* Remove unused development dependency.
* Add `Hash` trait to assorted metric types (`Counter`, `Timer`, `Gauge`, `Meter`).
* Documentation improvements.


## [v0.1.0](https://github.com/tshlabs/cadence/tree/0.1.0) - 2015-12-22

* Initial release.

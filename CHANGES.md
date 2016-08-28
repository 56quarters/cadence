# Changelog

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

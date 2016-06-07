# Changelog

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

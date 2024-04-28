# Migrations

Guides for migrating to different versions of Cadence are below.

## Migrating to 1.4.0

There are no backwards incompatible changes in this release.

## Migrating to 1.3.0

There are no backwards incompatible changes in this release.

## Migrating to 1.2.0

There are no backwards incompatible changes in this release.

## Migrating to 1.1.0

There are no backwards incompatible changes in this release.

## Migrating to 1.0.0

In version `1.0.0` of Cadence deprecated methods have been removed.

In particular:
* `StatsdClient::from_udp_host()` (deprecated since `0.19.0`) method has been removed. Instead,
  callers should use `StatsdClient::from_sink()` or `StatsdClient::builder()`.
* The `Compat` trait (deprecated since `0.26.0`), providing implementations of deprecated methods
  for `StatsdClient` has been removed. Callers should use the replacements detailed in the `0.26`
  migration notes.

## Migrating to 0.29

There are no backwards incompatible changes in this release.

## Migrating to 0.28

In version `0.28` of Cadence, support was added for packed values.
While this is a backwards incompatible change due to new variants of
the `MetricValue` type and extensions to the types supported by the
`MetricClient` trait, no changes are required for typical use of Cadence.

If you find this is not the case, please open an issue.

## Migrating to 0.27

In version `0.27` of Cadence, the `StatsdClient` struct no longer
implements the `Clone` trait. If you wish to clone instances of it
you must now wrap it with a container that can be cloned.

### Cloning using and `Arc`

```rust
use cadence::prelude::*;
use cadence::{NopMetricSink, StatsdClient};

fn main() {
    let client = Arc::new(StatsdClient::from_sink("some.prefix", NopMetricSink));
    let client_ref = client.clone();
    client_ref.count("some.counter", 123).unwrap();
}
```

## Migrating To 0.26

In version `0.26` of Cadence, the values for each type of metric are
generic in the methods to emit them. The implications of this and how
to update your code are discussed below.

### Generic metric values

Note the following example uses the `Timed` trait but this is applicable
to all traits for emitting metrics.

Methods to emit metrics changed to accept generic types:

```rust
pub trait Timed {
    fn time(&self, key: &str, time: u64) -> MetricResult<Timer>;
    fn time_duration(&self, key: &str, time: Duration) -> MetricResult<Timer>;
}
```
Becomes

```rust
pub trait Timed<T>
where
    T: ToTimerValue,
{
    fn time(&self, key: &str, time: T) -> MetricResult<Timer>;
}
```

To make sure your code works with the new trait, make sure the type
of your metric values can be unambiguously determined.

For example, give them explicit types

```rust
fn main() {
    let v: u64 = 42;
    client.time("some.key", v).unwrap();
}

```

Or cast them when passing to Cadence

```rust
fn main() {
    let v = 42;
    client.time("some.key", v as u64).unwrap();
}
```

### Moved methods

Due to technical requirements of making metric values generic, some methods
in the `Counted` trait needed to be moved into a new trait, `CountedExt`.

```rust
pub trait CountedExt: Counted<i64> {
    fn incr(&self, key: &str) -> MetricResult<Counter>;
    fn incr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<'_, '_, Counter>;
    fn decr(&self, key: &str) -> MetricResult<Counter>;
    fn decr_with_tags<'a>(&'a self, key: &'a str) -> MetricBuilder<'_, '_, Counter>;
}
```

If you glob-import the `cadence::prelude::*` module, this shouldn't require any
changes. If you _don't_ glob-import the prelude module, you'll need to add an
import for the `CountedExt` trait.

```rust
use cadence::CountedExt;
```

### Deprecated methods

Since type specific methods are no longer needed, they have been deprecated
and  moved to a new trait: `Compat`. Any methods that belong to this trait
will emit a deprecation warning when used. You should update your code to
use the suggested replacement for each deprecated method in this trait since
it will be removed in a future release of Cadence.

Deprecated methods and their replacements:

* `time_duration` -> `time`
* `time_duration_with_tags` -> `time_with_tags`
* `gauge_f64` -> `gauge`
* `gauge_f64_with_tags` -> `gauge_with_tags`
* `mark` -> `meter`
* `mark_with_tags` -> `meter_with_tags`
* `histogram_duration` -> `histogram`
* `histogram_duration_with_tags` -> `histogram_with_tags`


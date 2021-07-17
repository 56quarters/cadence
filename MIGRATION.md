# Migrations

Guides for migrating to different versions of Cadence are below.

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
let v: u64 = 42;
client.time("some.key", v).unwrap();
```

Or cast them when passing to Cadence

```rust
let v = 42;
client.time("some.key", v as u64).unwrap();
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


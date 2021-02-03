// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2020-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// NOTE: Comments here are mostly just copy/pasted. Make sure to update all of
//  them if you make changes!

/// Emit a counter using the default global client, optionally with tags
///
/// The counter will use the prefix from the default global client combined
/// with the provided key.
///
/// Any errors encountered sending metrics will be handled by the error handler
/// registered with the default global client. This error handler is a no-op
/// unless explicitly set. Callers should set the error handler for the default
/// client if you wish to handle these errors (by logging them or something similar).
///
/// # Panics
///
/// This macro will panic if the default global client has not been set when
/// it is invoked (via `cadence_macros::set_global_default`).
///
/// # Examples
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
/// use cadence_macros::statsd_count;
///
/// let client = StatsdClient::builder("my.prefix", NopMetricSink)
///     .with_error_handler(|e| { eprintln!("metric error: {}", e) })
///     .build();
///
/// cadence_macros::set_global_default(client);
///
/// // "my.prefix.some.counter:123|c"
/// statsd_count!("some.counter", 123);
/// // "my.prefix.some.counter:123|c|#tag:val"
/// statsd_count!("some.counter", 123, "tag" => "val");
/// // "my.prefix.some.counter:123|c|#tag:val,another:thing"
/// statsd_count!("some.counter", 123, "tag" => "val", "another" => "thing");
/// ```
///
/// # Limitations
///
/// Only key-value style tags are supported. Value style tags are not
/// supported, e.g. `builder.with_tag_value("val")`.
#[macro_export]
macro_rules! statsd_count {
    ($key:expr, $val:expr) => {
        $crate::statsd_count!($key, $val,)
    };

    ($key:expr, $val:expr, $($tag_key:expr => $tag_val:expr),*) => {
        $crate::_generate_impl!(count_with_tags, $key, $val, $($tag_key => $tag_val),*)
    }
}

/// Emit a timer using the default global client, optionally with tags
///
/// The timer will use the prefix from the default global client combined
/// with the provided key.
///
/// Any errors encountered sending metrics will be handled by the error handler
/// registered with the default global client. This error handler is a no-op
/// unless explicitly set. Callers should set the error handler for the default
/// client if you wish to handle these errors (by logging them or something similar).
///
/// # Panics
///
/// This macro will panic if the default global client has not been set when
/// it is invoked (via `cadence_macros::set_global_default`).
///
/// # Examples
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
/// use cadence_macros::statsd_time;
///
/// let client = StatsdClient::builder("my.prefix", NopMetricSink)
///     .with_error_handler(|e| { eprintln!("metric error: {}", e) })
///     .build();
///
/// cadence_macros::set_global_default(client);
///
/// // "my.prefix.some.timer:123|ms"
/// statsd_time!("some.timer", 123);
/// // "my.prefix.some.timer:123|ms|#tag:val"
/// statsd_time!("some.timer", 123, "tag" => "val");
/// // "my.prefix.some.timer:123|ms|#tag:val,another:thing"
/// statsd_time!("some.timer", 123, "tag" => "val", "another" => "thing");
/// ```
///
/// # Limitations
///
/// Only key-value style tags are supported. Value style tags are not
/// supported, e.g. `builder.with_tag_value("val")`.
#[macro_export]
macro_rules! statsd_time {
    ($key:expr, $val:expr) => {
        $crate::statsd_time!($key, $val,)
    };

    ($key:expr, $val:expr, $($tag_key:expr => $tag_val:expr),*) => {
        $crate::_generate_impl!(time_with_tags, $key, $val, $($tag_key => $tag_val),*)
    }
}

/// Emit a gauge using the default global client, optionally with tags
///
/// The gauge will use the prefix from the default global client combined
/// with the provided key.
///
/// Any errors encountered sending metrics will be handled by the error handler
/// registered with the default global client. This error handler is a no-op
/// unless explicitly set. Callers should set the error handler for the default
/// client if you wish to handle these errors (by logging them or something similar).
///
/// # Panics
///
/// This macro will panic if the default global client has not been set when
/// it is invoked (via `cadence_macros::set_global_default`).
///
/// # Examples
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
/// use cadence_macros::statsd_gauge;
///
/// let client = StatsdClient::builder("my.prefix", NopMetricSink)
///     .with_error_handler(|e| { eprintln!("metric error: {}", e) })
///     .build();
///
/// cadence_macros::set_global_default(client);
///
/// // "my.prefix.some.gauge:123|g"
/// statsd_gauge!("some.gauge", 123);
/// // "my.prefix.some.gauge:123|g|#tag:val"
/// statsd_gauge!("some.gauge", 123, "tag" => "val");
/// // "my.prefix.some.gauge:123|g|#tag:val,another:thing"
/// statsd_gauge!("some.gauge", 123, "tag" => "val", "another" => "thing");
/// ```
///
/// # Limitations
///
/// Only key-value style tags are supported. Value style tags are not
/// supported, e.g. `builder.with_tag_value("val")`.
#[macro_export]
macro_rules! statsd_gauge {
    ($key:expr, $val:expr) => {
        $crate::statsd_gauge!($key, $val,)
    };

    ($key:expr, $val:expr, $($tag_key:expr => $tag_val:expr),*) => {
        $crate::_generate_impl!(gauge_with_tags, $key, $val, $($tag_key => $tag_val),*)
    }
}

/// Emit a meter using the default global client, optionally with tags
///
/// The meter will use the prefix from the default global client combined
/// with the provided key.
///
/// Any errors encountered sending metrics will be handled by the error handler
/// registered with the default global client. This error handler is a no-op
/// unless explicitly set. Callers should set the error handler for the default
/// client if you wish to handle these errors (by logging them or something similar).
///
/// # Panics
///
/// This macro will panic if the default global client has not been set when
/// it is invoked (via `cadence_macros::set_global_default`).
///
/// # Examples
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
/// use cadence_macros::statsd_meter;
///
/// let client = StatsdClient::builder("my.prefix", NopMetricSink)
///     .with_error_handler(|e| { eprintln!("metric error: {}", e) })
///     .build();
///
/// cadence_macros::set_global_default(client);
///
/// // "my.prefix.some.meter:123|m"
/// statsd_meter!("some.meter", 123);
/// // "my.prefix.some.meter:123|m|#tag:val"
/// statsd_meter!("some.meter", 123, "tag" => "val");
/// // "my.prefix.some.meter:123|m|#tag:val,another:thing"
/// statsd_meter!("some.meter", 123, "tag" => "val", "another" => "thing");
/// ```
///
/// # Limitations
///
/// Only key-value style tags are supported. Value style tags are not
/// supported, e.g. `builder.with_tag_value("val")`.
#[macro_export]
macro_rules! statsd_meter {
    ($key:expr, $val:expr) => {
        $crate::statsd_meter!($key, $val,)
    };

    ($key:expr, $val:expr, $($tag_key:expr => $tag_val:expr),*) => {
        $crate::_generate_impl!(meter_with_tags, $key, $val, $($tag_key => $tag_val),*)
    }
}

/// Emit a histogram using the default global client, optionally with tags
///
/// The histogram will use the prefix from the default global client combined
/// with the provided key.
///
/// Any errors encountered sending metrics will be handled by the error handler
/// registered with the default global client. This error handler is a no-op
/// unless explicitly set. Callers should set the error handler for the default
/// client if you wish to handle these errors (by logging them or something similar).
///
/// # Panics
///
/// This macro will panic if the default global client has not been set when
/// it is invoked (via `cadence_macros::set_global_default`).
///
/// # Examples
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
/// use cadence_macros::statsd_histogram;
///
/// let client = StatsdClient::builder("my.prefix", NopMetricSink)
///     .with_error_handler(|e| { eprintln!("metric error: {}", e) })
///     .build();
///
/// cadence_macros::set_global_default(client);
///
/// // "my.prefix.some.histogram:123|h"
/// statsd_histogram!("some.histogram", 123);
/// // "my.prefix.some.histogram:123|h|#tag:val"
/// statsd_histogram!("some.histogram", 123, "tag" => "val");
/// // "my.prefix.some.histogram:123|h|#tag:val,another:thing"
/// statsd_histogram!("some.histogram", 123, "tag" => "val", "another" => "thing");
/// ```
///
/// # Limitations
///
/// Only key-value style tags are supported. Value style tags are not
/// supported, e.g. `builder.with_tag_value("val")`.
#[macro_export]
macro_rules! statsd_histogram {
    ($key:expr, $val:expr) => {
        $crate::statsd_histogram!($key, $val,)
    };

    ($key:expr, $val:expr, $($tag_key:expr => $tag_val:expr),*) => {
        $crate::_generate_impl!(histogram_with_tags, $key, $val, $($tag_key => $tag_val),*)
    }
}

/// Emit a set using the default global client, optionally with tags
///
/// The set will use the prefix from the default global client combined
/// with the provided key.
///
/// Any errors encountered sending metrics will be handled by the error handler
/// registered with the default global client. This error handler is a no-op
/// unless explicitly set. Callers should set the error handler for the default
/// client if you wish to handle these errors (by logging them or something similar).
///
/// # Panics
///
/// This macro will panic if the default global client has not been set when
/// it is invoked (via `cadence_macros::set_global_default`).
///
/// # Examples
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
/// use cadence_macros::statsd_set;
///
/// let client = StatsdClient::builder("my.prefix", NopMetricSink)
///     .with_error_handler(|e| { eprintln!("metric error: {}", e) })
///     .build();
///
/// cadence_macros::set_global_default(client);
///
/// // "my.prefix.some.set:123|s"
/// statsd_set!("some.set", 123);
/// // "my.prefix.some.set:123|s|#tag:val"
/// statsd_set!("some.set", 123, "tag" => "val");
/// // "my.prefix.some.set:123|s|#tag:val,another:thing"
/// statsd_set!("some.set", 123, "tag" => "val", "another" => "thing");
/// ```
///
/// # Limitations
///
/// Only key-value style tags are supported. Value style tags are not
/// supported, e.g. `builder.with_tag_value("val")`.
#[macro_export]
macro_rules! statsd_set {
    ($key:expr, $val:expr) => {
        $crate::statsd_set!($key, $val,)
    };

    ($key:expr, $val:expr, $($tag_key:expr => $tag_val:expr),*) => {
        $crate::_generate_impl!(set_with_tags, $key, $val, $($tag_key => $tag_val),*)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! _generate_impl {
    ($method:ident, $key:expr, $val:expr, $($tag_key:expr => $tag_val:expr),*) => {
        let client = $crate::get_global_default().unwrap();
        let builder = client.$method($key, $val);
        $(let builder = builder.with_tag($tag_key, $tag_val);)*
        builder.send()
    }
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2020 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use cadence::prelude::*;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Once};

type GlobalClient = dyn MetricClient + Send + Sync + RefUnwindSafe + 'static;

static GLOBAL_INIT: Once = Once::new();
static mut GLOBAL_DEFAULT: Option<Arc<GlobalClient>> = None;

/// Error indicating that a global default `MetricClient` was not set
/// when a call to `get_global_default` was made.
#[derive(Debug)]
pub struct GlobalDefaultNotSet;

impl Display for GlobalDefaultNotSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt("global default MetricClient instance not set", f)
    }
}

impl Error for GlobalDefaultNotSet {}

/// Set the global default `MetricClient` instance
///
/// If the global default client has already been set, this method does nothing.
///
/// # Example
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
/// let client = StatsdClient::from_sink("my.prefix", NopMetricSink);
///
/// cadence_macros::set_global_default(client);
/// ```
pub fn set_global_default<T>(client: T)
where
    T: MetricClient + Send + Sync + RefUnwindSafe + 'static,
{
    GLOBAL_INIT.call_once(move || {
        unsafe {
            GLOBAL_DEFAULT = Some(Arc::new(client));
        };
    });
}

/// Get a reference to the global default `MetricClient` instance
///
/// # Errors
///
/// This method will return an error if the global default has not been
/// previously set via the `set_global_default` method.
///
/// # Example
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
///
/// let global_client = cadence_macros::get_global_default();
/// assert!(global_client.is_err());
///
/// let client = StatsdClient::from_sink("my.prefix", NopMetricSink);
/// cadence_macros::set_global_default(client);
///
/// let global_client = cadence_macros::get_global_default();
/// assert!(global_client.is_ok());
/// ```
pub fn get_global_default() -> Result<Arc<GlobalClient>, GlobalDefaultNotSet> {
    unsafe { GLOBAL_DEFAULT.clone() }.ok_or(GlobalDefaultNotSet)
}

/// Return true if the global default `MetricClient` is set, false otherwise
///
/// # Example
///
/// ```
/// use cadence::{StatsdClient, NopMetricSink};
///
/// assert!(!cadence_macros::is_global_default_set());
///
/// let client = StatsdClient::from_sink("my.prefix", NopMetricSink);
/// cadence_macros::set_global_default(client);
///
/// assert!(cadence_macros::is_global_default_set());
/// ```
pub fn is_global_default_set() -> bool {
    // NOTE: not using Once::is_completed() here since it's rust 1.43+
    unsafe { &GLOBAL_DEFAULT }.is_some()
}

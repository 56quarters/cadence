// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2020-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use cadence::StatsdClient;
use std::cell::UnsafeCell;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const UNSET: usize = 0;
const LOADING: usize = 1;
const COMPLETE: usize = 2;

/// Global default StatsdClient to be used by macros
static HOLDER: SingletonHolder<StatsdClient> = SingletonHolder::new();

/// Holder to allow global reads of a value from multiple threads while
/// allowing the value to be written (set) a single time.
///
/// This type is public to allow it to be used in integration tests for
/// this crate but it is not part of the public API and may change at any
/// time.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct SingletonHolder<T> {
    value: UnsafeCell<Option<Arc<T>>>,
    state: AtomicUsize,
}

impl<T> SingletonHolder<T> {
    /// Create a new empty holder
    pub const fn new() -> Self {
        SingletonHolder {
            value: UnsafeCell::new(None),
            state: AtomicUsize::new(UNSET),
        }
    }
}

impl<T> SingletonHolder<T> {
    /// Get a pointer to the contained value if set, None otherwise
    pub fn get(&self) -> Option<Arc<T>> {
        if !self.is_set() {
            return None;
        }

        // SAFETY: We've ensured that the state is "complete" and the
        // set method has completed and set a value for the UnsafeCell.
        unsafe { &*self.value.get() }.clone()
    }

    pub fn is_set(&self) -> bool {
        COMPLETE == self.state.load(Ordering::Acquire)
    }

    /// Set the value if it has not already been set, otherwise this is a no-op
    pub fn set(&self, val: T) {
        if self
            .state
            .compare_exchange(UNSET, LOADING, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        // SAFETY: There are no readers at this point since we've guaranteed the
        // state could not have been "complete". There are no other writers since
        // we've ensured that the state was previously "unset" and we've been able
        // to compare-and-swap it to "loading".
        let ptr = self.value.get();
        unsafe {
            *ptr = Some(Arc::new(val));
        }

        self.state.store(COMPLETE, Ordering::Release);
    }
}

unsafe impl<T: Send> Send for SingletonHolder<T> {}

unsafe impl<T: Sync> Sync for SingletonHolder<T> {}

/// Error indicating that a global default `StatsdClient` was not set
/// when a call to `get_global_default` was made.
#[derive(Debug)]
pub struct GlobalDefaultNotSet;

impl Display for GlobalDefaultNotSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt("global default StatsdClient instance not set", f)
    }
}

impl Error for GlobalDefaultNotSet {}

/// Set the global default `StatsdClient` instance
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
pub fn set_global_default(client: StatsdClient) {
    HOLDER.set(client);
}

/// Get a reference to the global default `StatsdClient` instance
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
pub fn get_global_default() -> Result<Arc<StatsdClient>, GlobalDefaultNotSet> {
    HOLDER.get().ok_or(GlobalDefaultNotSet)
}

/// Return true if the global default `StatsdClient` is set, false otherwise
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
    HOLDER.is_set()
}

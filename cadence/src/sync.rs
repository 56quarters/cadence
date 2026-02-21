// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2026 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

/// Statistics about the job being run by the `execute` method.
#[derive(Debug, Default)]
pub struct ExecuteStats {
    panics: AtomicU64,
}

impl ExecuteStats {
    fn incr_panic(&self) {
        self.panics.fetch_add(1, Ordering::Relaxed);
    }

    pub fn panics(&self) -> u64 {
        self.panics.load(Ordering::Relaxed)
    }
}

/// Execute the task `f` in a new thread until completion, restarting it if the
/// task panics.
///
/// Since a new thread is created for each task executed, this should only be used for
/// long-running tasks.
pub fn execute<F>(f: F) -> Arc<ExecuteStats>
where
    F: Fn() + Send + Sync + RefUnwindSafe + 'static,
{
    let stats = Arc::new(ExecuteStats::default());
    spawn_in_thread(Arc::new(f), stats.clone());
    stats
}

/// Create a thread and run the job in it to completion
///
/// This function uses a `Sentinel` struct to make sure that any panics from
/// running the job result in another thread being spawned to start running
/// the job again.
fn spawn_in_thread<F>(job: Arc<F>, metrics: Arc<ExecuteStats>) -> thread::JoinHandle<()>
where
    F: Fn() + Send + Sync + RefUnwindSafe + 'static,
{
    thread::spawn(move || {
        let mut sentinel = Sentinel::new(&job, &metrics);
        job();
        sentinel.cancel();
    })
}

/// Struct for ensuring a worker runs to completion correctly, without
/// panicking.
///
/// The sentinel will spawn a new thread to continue running the worker
/// in its destructor unless the `.cancel()` method is called after the
/// worker completes (which won't happen if the worker panics).
#[derive(Debug)]
struct Sentinel<'a, F>
where
    F: Fn() + Send + Sync + RefUnwindSafe + 'static,
{
    job: &'a Arc<F>,
    stats: &'a Arc<ExecuteStats>,
    active: bool,
}

impl<'a, F> Sentinel<'a, F>
where
    F: Fn() + Send + Sync + RefUnwindSafe + 'static,
{
    fn new(job: &'a Arc<F>, stats: &'a Arc<ExecuteStats>) -> Sentinel<'a, F> {
        Sentinel {
            job,
            stats,
            active: true,
        }
    }

    fn cancel(&mut self) {
        self.active = false;
    }
}

impl<'a, F> Drop for Sentinel<'a, F>
where
    F: Fn() + Send + Sync + RefUnwindSafe + 'static,
{
    fn drop(&mut self) {
        if self.active {
            // This sentinel didn't have its `.cancel()`method called so
            // the thread must have panicked. Increment a counter indicating
            // that this was a panic and spawn a new thread with an Arc of
            // the worker.
            self.stats.incr_panic();
            spawn_in_thread(self.job.clone(), self.stats.clone());
        }
    }
}

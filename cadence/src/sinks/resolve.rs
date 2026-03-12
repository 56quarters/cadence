// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2026 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::types::{ErrorKind, MetricError, MetricResult};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Duration;
use std::{fmt, io};

/// Attempt to convert anything implementing the `ToSocketAddrs` trait
/// into a concrete `SocketAddr` instance, returning an `InvalidInput`
/// error if the address could not be parsed.
fn get_addr<A: ToSocketAddrs>(addr: &A) -> MetricResult<SocketAddr> {
    match addr.to_socket_addrs()?.next() {
        Some(addr) => Ok(addr),
        None => Err(MetricError::from((
            ErrorKind::InvalidInput,
            "No socket addresses yielded",
        ))),
    }
}

/// Trait to get a `SocketAddr` to send metrics to, used by a
/// `MetricSink`.
///
/// Different implementations allow different behavior such as only
/// resolving hostnames when constructed or periodically re-resolving
/// them using DNS.
pub(crate) trait Resolver {
    /// Get a `SocketAddr` for a metrics server.
    ///
    /// Implementations are expected _not_ to resolve hostnames as part of
    /// this method call, that should be done out-of-band.
    fn get_addr(&self) -> SocketAddr;

    /// Signal that any background tasks started by the `Resolver` should
    /// stop.
    fn stop(&self) {}
}

/// `Resolver` implementation that does address resolution on construction
/// only.
#[derive(Debug)]
pub(crate) struct StaticResolver {
    addr: SocketAddr,
}

impl StaticResolver {
    pub fn new<A>(name: A) -> MetricResult<Self>
    where
        A: ToSocketAddrs,
    {
        let addr = get_addr(&name)?;
        Ok(Self { addr })
    }
}

impl Resolver for StaticResolver {
    fn get_addr(&self) -> SocketAddr {
        self.addr
    }
}

/// `Resolver` implementation that does address resolution on construction
/// and periodically in the background in a separate thread.
///
/// The resolver does not create the thread for performing resolution in the
/// background, the caller is expected to spawn the thread and run the `run`
/// method in it. The `run` method will run until the `stop` method of the
/// resolver is called.
pub(crate) struct PeriodicResolver<A, E, S>
where
    A: ToSocketAddrs + fmt::Debug,
    E: Fn(io::Error),
    S: Fn(Duration),
{
    name: A,
    errors: E,
    sleep: S,
    addr: RwLock<SocketAddr>,
    period: Duration,
    run: AtomicBool,
    stopped: AtomicBool,
    successes: AtomicU64,
    failures: AtomicU64,
}

impl<A, E, S> PeriodicResolver<A, E, S>
where
    A: ToSocketAddrs + fmt::Debug,
    E: Fn(io::Error),
    S: Fn(Duration),
{
    pub(crate) fn new(name: A, period: Duration, errors: E, sleep: S) -> MetricResult<Self> {
        let addr = get_addr(&name)?;

        Ok(Self {
            addr: RwLock::new(addr),
            run: AtomicBool::new(true),
            stopped: AtomicBool::new(false),
            successes: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            name,
            errors,
            sleep,
            period,
        })
    }

    pub(crate) fn run(&self) {
        while self.run.load(Ordering::Acquire) {
            (self.sleep)(self.period);

            let addr = match self.name.to_socket_addrs().map(|mut i| i.next()) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    self.incr_failures();
                    (self.errors)(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("{:?} did not resolve to any addresses", self.name),
                    ));
                    continue;
                }
                Err(e) => {
                    self.incr_failures();
                    (self.errors)(e);
                    continue;
                }
            };

            self.incr_success();
            *self.addr.write().unwrap() = addr;
        }

        self.stopped.store(true, Ordering::Release);
    }

    fn incr_success(&self) -> u64 {
        self.successes.fetch_add(1, Ordering::Relaxed)
    }

    #[cfg(test)]
    fn successes(&self) -> u64 {
        self.successes.load(Ordering::Relaxed)
    }

    fn incr_failures(&self) -> u64 {
        self.failures.fetch_add(1, Ordering::Relaxed)
    }

    #[cfg(test)]
    fn failures(&self) -> u64 {
        self.failures.load(Ordering::Relaxed)
    }
}

impl<A, E, S> fmt::Debug for PeriodicResolver<A, E, S>
where
    A: ToSocketAddrs + fmt::Debug,
    E: Fn(io::Error),
    S: Fn(Duration),
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeriodicResolver")
            .field("name", &self.name)
            .field("period", &self.period)
            .finish()
    }
}

impl<A, E, S> Resolver for PeriodicResolver<A, E, S>
where
    A: ToSocketAddrs + fmt::Debug,
    E: Fn(io::Error),
    S: Fn(Duration),
{
    fn get_addr(&self) -> SocketAddr {
        *self.addr.read().unwrap()
    }

    fn stop(&self) {
        self.run.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::{PeriodicResolver, Resolver};
    use crate::types::ErrorKind;
    use std::net::{Ipv4Addr, SocketAddr, ToSocketAddrs};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use std::{io, thread, vec};

    #[test]
    fn test_periodic_resolver_initial_resolution_fails() {
        let err = PeriodicResolver::new("invalid:invalid", Duration::from_secs(1), |_e| {}, |_d| {}).unwrap_err();
        assert_eq!(ErrorKind::IoError, err.kind());
    }

    #[test]
    fn test_periodic_resolver_initial_resolution_succeeds() {
        let resolver = PeriodicResolver::new("127.0.0.1:8125", Duration::from_secs(1), |_e| {}, |_d| {}).unwrap();
        let expected = SocketAddr::from((Ipv4Addr::new(127, 0, 0, 1), 8125));
        assert_eq!(expected, resolver.get_addr());
    }

    #[derive(Debug)]
    struct FailingAddr {
        name: String,
        successes: u64,
        count: AtomicU64,
    }

    impl FailingAddr {
        fn new<S>(name: S, successes: u64) -> Self
        where
            S: Into<String>,
        {
            Self {
                name: name.into(),
                count: AtomicU64::new(0),
                successes,
            }
        }
    }

    impl ToSocketAddrs for FailingAddr {
        type Iter = vec::IntoIter<SocketAddr>;

        fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
            self.count.fetch_add(1, Ordering::Relaxed);

            if self.count.load(Ordering::Relaxed) <= self.successes {
                self.name.to_socket_addrs()
            } else {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "test lookup failed"))
            }
        }
    }

    #[test]
    fn test_periodic_resolver_periodic_resolution_fails() {
        // Zero capacity channels so that send/recv calls block until the corresponding
        // call is made in another thread. This allows us to enter the main "run" loop,
        // pause, and then only proceed after the "run" loop has been stopped from another
        // thread ensuring we get a single iteration.
        let (run_tx, run_rx) = crossbeam_channel::bounded(0);
        let (done_tx, done_rx) = crossbeam_channel::bounded(0);

        let sleep = move |_d: Duration| {
            run_rx.recv().unwrap();
            done_tx.send(()).unwrap();
        };

        let error_count = Arc::new(AtomicU64::new(0));
        let error_count_c = error_count.clone();
        let errors = move |_e| {
            error_count_c.fetch_add(1, Ordering::Relaxed);
        };

        // This address will return results for the first (initial) call to
        // `to_socket_addrs` but fail during the periodic re-resolve that the
        // resolver attempts.
        let addr = FailingAddr::new("127.0.0.1:8125", 1);

        let resolver = Arc::new(PeriodicResolver::new(addr, Duration::from_secs(1), errors, sleep).unwrap());
        let resolver_c = resolver.clone();

        let t1 = thread::spawn(move || {
            run_tx.send(()).unwrap();
            resolver_c.stop();
            done_rx.recv().unwrap();
        });

        resolver.run();
        let _ = t1.join();

        let expected = SocketAddr::from((Ipv4Addr::new(127, 0, 0, 1), 8125));
        assert_eq!(expected, resolver.get_addr());
        assert_eq!(0, resolver.successes());
        assert_eq!(1, resolver.failures());
        assert_eq!(1, error_count.load(Ordering::Relaxed));
    }

    #[test]
    fn test_periodic_resolver_periodic_resolution_succeeds() {
        // Zero capacity channels so that send/recv calls block until the corresponding
        // call is made in another thread. This allows us to enter the main "run" loop,
        // pause, and then only proceed after the "run" loop has been stopped from another
        // thread ensuring we get a single iteration.
        let (run_tx, run_rx) = crossbeam_channel::bounded(0);
        let (done_tx, done_rx) = crossbeam_channel::bounded(0);

        let sleep = move |_d: Duration| {
            run_rx.recv().unwrap();
            done_tx.send(()).unwrap();
        };

        let resolver =
            Arc::new(PeriodicResolver::new("127.0.0.1:8125", Duration::from_secs(1), |_e| {}, sleep).unwrap());
        let resolver_c1 = resolver.clone();

        let t1 = thread::spawn(move || {
            run_tx.send(()).unwrap();
            resolver_c1.stop();
            done_rx.recv().unwrap();
        });

        resolver.run();
        let _ = t1.join();

        let expected = SocketAddr::from((Ipv4Addr::new(127, 0, 0, 1), 8125));
        assert_eq!(expected, resolver.get_addr());
        assert_eq!(1, resolver.successes());
        assert_eq!(0, resolver.failures());
    }
}

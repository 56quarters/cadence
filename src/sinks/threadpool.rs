// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use threadpool::ThreadPool;

use std::io;
use std::sync::Arc;

use ::sinks::MetricSink;


// Default size (number of threads) for the thread pool used by
// the `AsyncMetricSink` for sending metrics. This value is rather
// arbitrary but should be fine for most use cases. Users that
// need further customization can use the alternate constructor
// for the `AsyncMetricSink`.
const DEFAULT_THREAD_POOL_SIZE: usize = 4;


// Default name of the threads in the thread pool used by the
// `AsyncMetricSink`.
const DEFAULT_THREAD_POOL_NAME: &'static str = "cadence";


/// Implementation of a `MetricSink` that wraps another implementation
/// and uses it to emit metrics asynchronously with a thread pool.
///
/// The wrapped implementation can by any thread safe (`Send + Sync`)
/// `MetricSink` implementation. Results from the wrapped implementation
/// will be discarded.
///
/// Because this `MetricSink` implementation uses a thread pool, the sink
/// itself cannot be shared between threads. Instead, callers may opt to
/// create a `.clone()` for each thread that needs to emit metrics. This
/// of course requires that the wrapped sink implements the `Clone` trait
/// (all of the sinks that are part of Cadence implement `Clone`).
///
/// When cloned, the new instance of this sink will have a cloned thread
/// pool instance that submits jobs to the same worker threads as the
/// original pool and a reference (`Arc`) to the wrapped sink. If you
/// plan on cloning this sink, the thread pool should be sized
/// appropriately to be used by all expected sink instances.
#[derive(Debug, Clone)]
#[deprecated(since="0.10.0", note="Replaced with QueuingMetricSink. This \
                                   will be removed in version 0.11.0")]
pub struct AsyncMetricSink<T: 'static + MetricSink + Send + Sync> {
    pool: ThreadPool,
    delegate: Arc<T>,
}


impl<T: 'static + MetricSink + Send + Sync> AsyncMetricSink<T> {
    /// Construct a new `AsyncMetricSink` instance wrapping another sink
    /// implementation.
    ///
    /// The `.emit()` method of the wrapped sink will be executed in a
    /// different thread via a thread pool. The wrapped sink should be
    /// thread safe (`Send + Sync`).
    ///
    /// The default thread pool size is four threads. Callers can use
    /// more or fewer threads by making use of the `with_threadpool`
    /// constructor.
    ///
    /// # UDP Sink Example
    ///
    /// In this example we wrap the basic UDP sink to execute it in a
    /// different thread.
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{UdpMetricSink, AsyncMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let udp_sink = UdpMetricSink::from(host, socket).unwrap();
    /// let async_sink = AsyncMetricSink::from(udp_sink);
    /// ```
    ///
    /// # Buffered UDP Sink Example
    ///
    /// This example uses the buffered UDP sink, wrapped in the async
    /// metric sink.
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{BufferedUdpMetricSink, AsyncMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    /// let async_sink = AsyncMetricSink::from(udp_sink);
    /// ```
    pub fn from(sink: T) -> AsyncMetricSink<T> {
        Self::with_threadpool(sink, ThreadPool::new_with_name(
            DEFAULT_THREAD_POOL_NAME.to_string(),
            DEFAULT_THREAD_POOL_SIZE
        ))
    }

    /// Construct a new `AsyncMetricSink` instance wrapping another sink
    /// implementation that will use the provided thread pool for
    /// asynchronous execution.
    ///
    /// The `.emit()` method of the wrapped sink will be executed in a
    /// different thread via a thread pool. The wrapped sink should be
    /// thread safe (`Send + Sync`).
    ///
    /// # Buffered UDP Sink With a Single Threaded Pool
    ///
    /// ```no_run
    /// # extern crate threadpool;
    /// # extern crate cadence;
    /// # fn main() {
    /// use std::net::UdpSocket;
    /// use threadpool::ThreadPool;
    /// use cadence::{BufferedUdpMetricSink, AsyncMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    /// let async_sink = AsyncMetricSink::with_threadpool(
    ///     udp_sink, ThreadPool::new(1));
    /// # }
    /// ```
    pub fn with_threadpool(sink: T, pool: ThreadPool) -> AsyncMetricSink<T> {
        AsyncMetricSink {
            pool: pool,
            delegate: Arc::new(sink),
        }
    }
}


impl<T: 'static + MetricSink + Send + Sync> MetricSink for AsyncMetricSink<T> {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        let owned_metric = metric.to_string();
        let sink = self.delegate.clone();

        self.pool.execute(move || {
            let _r = sink.emit(&owned_metric);
        });

        Ok(metric.len())
    }
}


#[cfg(test)]
mod tests {
    use ::sinks::{MetricSink, NopMetricSink};
    use super::AsyncMetricSink;

    #[test]
    fn test_async_nop_metric_sink() {
        let sink = AsyncMetricSink::from(NopMetricSink);
        assert_eq!(8, sink.emit("buz:33|c").unwrap());
        assert_eq!(8, sink.emit("boo:27|c").unwrap());
    }
}

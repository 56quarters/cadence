// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::sinks::core::{MetricSink, SinkStats};
use crossbeam_channel::{self, Receiver, Sender, TrySendError};
use std::fmt;
use std::io::{self, ErrorKind};
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

/// Implementation of a builder pattern for `QueuingMetricSink`.
///
/// The builder can be used to set an error handler for the sink being
/// wrapped by a `QueuingMetricSink` as well as setting the capacity of
/// the `QueuingMetricSink`.
///
/// # Example
///
/// ```no_run
/// use cadence::{MetricSink, QueuingMetricSinkBuilder, NopMetricSink};
///
/// let queue_size = 64 * 1024;
/// let wrapped = NopMetricSink;
///
/// let queuing = QueuingMetricSinkBuilder::new()
///     .with_capacity(queue_size)
///     .with_error_handler(|e| {
///         eprintln!("Error while sending metrics: {:?}", e);
///     })
///     .build(wrapped);
///
/// queuing.emit("foo.counter:4|c");
/// ```
#[derive(Default)]
pub struct QueuingMetricSinkBuilder {
    error_handler: Option<Box<dyn Fn(io::Error) + Sync + Send + RefUnwindSafe + 'static>>,
    capacity: Option<usize>,
}

impl QueuingMetricSinkBuilder {
    /// Construct a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a new `QueuingMetricSink` instance wrapping another sink based on
    /// the builder configuration.
    pub fn build<T>(self, sink: T) -> QueuingMetricSink
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        let sink = Arc::new(sink);
        let sink_c = sink.clone();
        let worker = Arc::new(Worker::new(self.capacity, move |v: String| {
            if let Err(e) = sink_c.emit(&v) {
                if let Some(error_handler) = &self.error_handler {
                    error_handler(e);
                }
            }
        }));

        spawn_worker_in_thread(worker.clone());

        QueuingMetricSink { worker, sink }
    }

    /// Set error handler called when the wrapped sink fails to emit a metric.
    ///
    /// The error handler will be run in the same thread as the wrapped sink and
    /// must not panic.
    pub fn with_error_handler<F>(mut self, error_handler: F) -> Self
    where
        F: Fn(io::Error) + Sync + Send + RefUnwindSafe + 'static,
    {
        self.error_handler = Some(Box::new(error_handler));
        self
    }

    /// Set queue size used to send metrics to the wrapped sink.
    ///
    /// See `QueuingMetricSink::with_capacity` for more information.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }
}

/// Implementation of a `MetricSink` that wraps another implementation
/// and uses it to emit metrics asynchronously, in another thread.
///
/// Metrics submitted to this sink are queued and sent to the wrapped sink
/// that is running in a separate thread. The wrapped implementation can
/// be any thread (`Sync` + `Send`) and panic (`RefUnwindSafe`) safe
/// `MetricSink`. Results from the wrapped implementation will be discarded.
///
/// The thread used for network operations (actually sending the metrics
/// using the wrapped sink) is created and started when the `QueuingMetricSink`
/// is created. The dequeuing of metrics is stopped and the thread stopped
/// when `QueuingMetricSink` instance is destroyed (when `.drop()` is
/// called).
///
/// This sink may be created with either a bounded or unbounded queue
/// connecting the sink to the thread performing network operations. When an
/// unbounded queue is used, entries submitted to the sink will always be
/// accepted and queued until they can be drained by the network operation
/// thread. This means that if the network thread cannot drain entries off
/// the queue for some reason, it will grow without bound. Alternatively, if
/// created with a bounded queue, entries submitted to the sink will not be
/// accepted if the queue is full. This means that the network thread must
/// be able to keep up with the rate of entries submit to the queue or writes
/// to this sink will begin to fail.
///
/// Entries already queued are guaranteed to be sent to the wrapped sink
/// before the queuing sink is stopped. Meaning, the following code ends up
/// calling `wrapped.emit(metric)` on every metric submitted to the queuing
/// sink.
///
/// # Example
///
/// ```no_run
/// use cadence::{MetricSink, QueuingMetricSink, NopMetricSink};
///
/// let wrapped = NopMetricSink;
/// {
///     let queuing = QueuingMetricSink::from(wrapped);
///     queuing.emit("foo.counter:4|c");
///     queuing.emit("bar.counter:5|c");
///     queuing.emit("baz.gauge:6|g");
/// }
/// ```
///
/// At the end of this code block, all metrics are guaranteed to be sent to
/// the underlying wrapped metric sink before the thread used by the queuing
/// sink is stopped.
#[derive(Clone)]
pub struct QueuingMetricSink {
    worker: Arc<Worker>,
    sink: Arc<dyn MetricSink + Send + Sync + RefUnwindSafe>,
}

impl fmt::Debug for QueuingMetricSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QueuingMetricSink {{ {:?} }}", self.worker)
    }
}

impl QueuingMetricSink {
    /// Construct a new builder for `QueuingMetricSink`.
    pub fn builder() -> QueuingMetricSinkBuilder {
        QueuingMetricSinkBuilder::new()
    }

    /// Construct a new `QueuingMetricSink` instance wrapping another sink
    /// implementation with an unbounded queue connecting them.
    ///
    /// The `.emit()` method of the wrapped sink will be executed in a
    /// different thread after being passed to it via a queue. The wrapped
    /// sink should be thread safe (`Send + Sync`) and panic safe
    /// (`RefUnwindSafe`).
    ///
    /// The thread in which the wrapped sink runs is created when the
    /// `QueuingMetricSink` is created and stopped when the queuing sink
    /// is destroyed.
    ///
    /// The queuing sink communicates with the wrapped sink by an unbounded
    /// queue. If entries cannot be drained from the queue for some reason, it
    /// will grow without bound.
    ///
    /// # Buffered UDP Sink Example
    ///
    /// In this example we wrap a buffered UDP sink to execute it in a
    /// different thread.
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{BufferedUdpMetricSink, QueuingMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    /// let queuing_sink = QueuingMetricSink::from(udp_sink);
    /// ```
    pub fn from<T>(sink: T) -> Self
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        Self::builder().build(sink)
    }

    /// Construct a new `QueuingMetricSink` instance wrapping another sink
    /// implementation with a queue of the given size connecting them.
    ///
    /// The `.emit()` method of the wrapped sink will be executed in a
    /// different thread after being passed to it via a queue. The wrapped
    /// sink should be thread safe (`Send + Sync`) and panic safe
    /// (`RefUnwindSafe`).
    ///
    /// The thread in which the wrapped sink runs is created when the
    /// `QueuingMetricSink` is created and stopped when the queuing sink
    /// is destroyed.
    ///
    /// The queuing sink communicates with the wrapped sink by a bounded
    /// queue of the provided size. When the queue is full, writes to
    /// this sink will fail until the queue is drained.
    ///
    /// # Buffered UDP Sink Example
    ///
    /// In this example we wrap a buffered UDP sink to execute it in a
    /// different thread.
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{BufferedUdpMetricSink, QueuingMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    /// let queuing_sink = QueuingMetricSink::with_capacity(udp_sink, 512 * 1024);
    /// ```
    pub fn with_capacity<T>(sink: T, capacity: usize) -> Self
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        Self::builder().with_capacity(capacity).build(sink)
    }

    /// Return the number of times the wrapped sink or underlying worker thread
    /// has panicked and needed to be restarted. In typical use this should always
    /// be `0` but may be `> 0` for buggy `MetricSink` implementations.
    pub fn panics(&self) -> u64 {
        self.worker.stats.panics()
    }

    /// Return the number of currently queued metrics. Note that due to the way
    /// this number is computed (submitted metrics - processed metrics), it is
    /// necessarily approximate.
    pub fn queued(&self) -> u64 {
        self.worker.stats.queued()
    }

    /// Return the number of metrics successfully submitted to this sink.
    pub fn submitted(&self) -> u64 {
        self.worker.stats.submitted()
    }

    /// Return the number of metrics removed from the queue to be processed by
    /// the wrapped sink. Note that this does not indicate that the metric has
    /// been successfully sent to a backend, only that it has been passed to
    /// the wrapped sink.
    pub fn drained(&self) -> u64 {
        self.worker.stats.drained()
    }
}

impl MetricSink for QueuingMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        // Required while MSRV is 1.60
        // std::io::Error::other stabilized in 1.74
        #[allow(unknown_lints, clippy::io_other_error)]
        match self.worker.submit(metric.to_string()) {
            Err(TrySendError::Disconnected(_)) => Err(io::Error::new(ErrorKind::Other, "channel disconnected")),
            Err(TrySendError::Full(_)) => Err(io::Error::new(ErrorKind::Other, "channel full")),
            Ok(_) => Ok(metric.len()),
        }
    }

    fn flush(&self) -> Result<(), std::io::Error> {
        self.sink.flush()
    }

    fn stats(&self) -> SinkStats {
        self.sink.stats()
    }
}

impl Drop for QueuingMetricSink {
    /// Send the worker a signal to stop processing metrics.
    ///
    /// Note that this destructor only sends the worker thread a signal to
    /// stop, it doesn't wait for it to stop.
    fn drop(&mut self) {
        self.worker.stop();
    }
}

/// Statistics about the worker running.
///
/// These statistics are only used for unit testing to verify that our
/// sentinel can handle thread panics and restart the thread the worker
/// is running in.
#[derive(Debug)]
struct WorkerStats {
    panics: AtomicU64,
    submitted: AtomicU64,
    drained: AtomicU64,
}

impl WorkerStats {
    fn new() -> WorkerStats {
        WorkerStats {
            panics: AtomicU64::new(0),
            submitted: AtomicU64::new(0),
            drained: AtomicU64::new(0),
        }
    }

    fn incr_panic(&self) {
        self.panics.fetch_add(1, Ordering::Release);
    }

    fn panics(&self) -> u64 {
        self.panics.load(Ordering::Acquire)
    }

    fn incr_submitted(&self) {
        self.submitted.fetch_add(1, Ordering::Release);
    }

    fn submitted(&self) -> u64 {
        self.submitted.load(Ordering::Acquire)
    }

    fn incr_drained(&self) {
        self.drained.fetch_add(1, Ordering::Release);
    }

    fn drained(&self) -> u64 {
        self.drained.load(Ordering::Acquire)
    }

    fn queued(&self) -> u64 {
        let submitted = self.submitted.load(Ordering::Acquire);
        let drained = self.drained.load(Ordering::Acquire);

        submitted.saturating_sub(drained)
    }
}

/// Create a thread and run the worker in it to completion
///
/// This function uses a `Sentinel` struct to make sure that any panics from
/// running the worker result in another thread being spawned to start running
/// the worker again.
fn spawn_worker_in_thread(worker: Arc<Worker>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut sentinel = Sentinel::new(&worker);
        worker.run();
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
struct Sentinel<'a> {
    worker: &'a Arc<Worker>,
    active: bool,
}

impl<'a> Sentinel<'a> {
    fn new(worker: &'a Arc<Worker>) -> Sentinel<'a> {
        Sentinel { worker, active: true }
    }

    fn cancel(&mut self) {
        self.active = false;
    }
}

impl<'a> Drop for Sentinel<'a> {
    fn drop(&mut self) {
        if self.active {
            // This sentinel didn't have its `.cancel()`method called so
            // the thread must have panicked. Increment a counter indicating
            // that this was a panic and spawn a new thread with an Arc of
            // the worker.
            self.worker.stats.incr_panic();
            spawn_worker_in_thread(self.worker.clone());
        }
    }
}

/// Worker to repeatedly run a method consuming entries via a channel.
///
/// The `.run()` method of the worker is intended to be in a separate
/// thread (thread B). Meanwhile, the `.submit()`, `.stop()`,
/// `.stop_and_wait()`, and `.is_stopped()` methods are meant to be called
/// from the main thread (thread A).
///
/// This worker is stopped by receiving a "poison pill" message in the
/// channel that it is consuming messages from. Thus, calls to `.submit()`,
/// consuming messages in '.run()`, and `.stop()` typically involve no
/// locking.
///
/// However, in order to enable easier testing, after it stops receiving
/// messages the `.run()` method will use an atomic "stopped" flag to
/// allow callers waiting on a conditional variable (callers using
/// `.stop_and_wait()`) to wake up after the worker finally stops.
///
/// If you're just trying to make use of this worker you don't need to
/// worry about this, just call `.submit()`, `.run()`, and `.stop()`.
/// But, if you're wondering why the stopped flag and methods to wait
/// for it or inspect it even exist: testing is the reason.
struct Worker {
    task: Box<dyn Fn(String) + Sync + Send + RefUnwindSafe + 'static>,
    sender: Sender<Option<String>>,
    receiver: Receiver<Option<String>>,
    stopped: AtomicBool,
    stats: WorkerStats,
}

impl Worker {
    fn new<F>(capacity: Option<usize>, task: F) -> Self
    where
        F: Fn(String) + Sync + Send + RefUnwindSafe + 'static,
    {
        let (tx, rx) = Self::get_channels(capacity);
        Worker {
            task: Box::new(task),
            sender: tx,
            receiver: rx,
            stopped: AtomicBool::new(false),
            stats: WorkerStats::new(),
        }
    }

    fn get_channels(capacity: Option<usize>) -> (Sender<Option<String>>, Receiver<Option<String>>) {
        if let Some(v) = capacity {
            crossbeam_channel::bounded(v)
        } else {
            crossbeam_channel::unbounded()
        }
    }

    fn submit(&self, v: String) -> Result<(), TrySendError<Option<String>>> {
        let res = self.sender.try_send(Some(v));
        if res.is_ok() {
            self.stats.incr_submitted();
        }

        res
    }

    fn run(&self) {
        for opt in self.receiver.iter() {
            if let Some(v) = opt {
                self.stats.incr_drained();
                (self.task)(v);
            } else {
                break;
            }
        }

        // Set the "stopped" flag so that callers using the `stop_and_wait`
        // method will see that we've stopped processing entries in the channel.
        // This is only for the benefit of unit testing.
        self.stopped.store(true, Ordering::Release);
    }

    fn stop(&self) {
        // Send a `None` poison pill value to stop the run loop.
        let _ = self.sender.try_send(None);
    }

    // Stop reading events from the channel and wait for the "stopped" flag
    // to be set. Note that this repeatedly yields the current thread and is
    // only intended for unit testing.
    #[cfg(test)]
    fn stop_and_wait(&self) {
        self.stop();

        while !self.stopped.load(Ordering::Acquire) {
            thread::yield_now();
        }
    }

    // Is the channel used between threads empty, i.e. are all values processed?
    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }

    // Has this worker stopped running?
    #[cfg(test)]
    fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}

impl fmt::Debug for Worker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Worker {{ ... }}")
    }
}

#[cfg(test)]
mod tests {
    use super::{QueuingMetricSink, Worker};
    use crate::sinks::MetricSink;
    use crate::sinks::SpyMetricSink;
    use crate::test::PanickingMetricSink;
    use std::io;
    use std::panic;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;

    const QUEUE_SIZE: Option<usize> = Some(128);

    #[test]
    fn test_worker_submit_processes_event() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_ref = flag.clone();

        let task = move |v: String| {
            if v == "foo" {
                flag_ref.store(true, Ordering::Release);
            }
        };

        let worker = Arc::new(Worker::new(QUEUE_SIZE, task));
        let worker_ref = worker.clone();

        let t = thread::spawn(move || {
            worker_ref.run();
        });

        worker.submit("bar".to_string()).unwrap();
        worker.submit("foo".to_string()).unwrap();
        worker.stop();
        t.join().unwrap();

        assert!(flag.load(Ordering::Acquire));
    }

    #[test]
    fn test_worker_stop() {
        let worker = Arc::new(Worker::new(QUEUE_SIZE, move |_: String| {}));
        let worker_ref = worker.clone();

        let t = thread::spawn(move || {
            worker_ref.run();
        });

        worker.stop();
        t.join().unwrap();

        assert!(worker.is_stopped());
    }

    #[test]
    fn test_worker_stop_and_wait() {
        let worker = Arc::new(Worker::new(QUEUE_SIZE, move |_: String| {}));
        let worker_ref = worker.clone();

        let _t = thread::spawn(move || {
            worker_ref.run();
        });

        worker.stop_and_wait();
        assert!(worker.is_stopped());
    }

    // Make sure the worker and its channel are in the expected state
    // when the producer size of the channel panics.
    #[test]
    fn test_worker_panic_on_submit_side() {
        let worker = Arc::new(Worker::new(QUEUE_SIZE, move |_: String| {}));
        let worker_ref1 = worker.clone();
        let worker_ref2 = worker.clone();

        #[allow(unreachable_code)]
        #[allow(clippy::diverging_sub_expression)]
        let t1 = thread::spawn(move || {
            worker_ref1.submit(panic!("This thread is supposed to panic")).unwrap();
        });

        let t2 = thread::spawn(move || {
            worker_ref2.run();
        });

        worker.stop();

        assert!(t1.join().is_err());
        assert!(t2.join().is_ok());

        assert!(worker.is_stopped());
        assert!(worker.is_empty());
    }

    // Make sure the worker and its channel are in the expected state
    // when the consumer side of the channel panics.
    #[test]
    fn test_worker_panic_on_run_side() {
        let worker = Arc::new(Worker::new(QUEUE_SIZE, move |_: String| {
            panic!("This thread is supposed to panic");
        }));
        let worker_ref1 = worker.clone();
        let worker_ref2 = worker.clone();

        let t1 = thread::spawn(move || {
            worker_ref1.submit("foo".to_owned()).unwrap();
        });

        let t2 = thread::spawn(move || {
            worker_ref2.run();
        });

        assert!(t1.join().is_ok());
        assert!(t2.join().is_err());

        assert!(!worker.is_stopped());
        assert!(worker.is_empty());
    }

    #[test]
    fn test_queuing_sink_emit() {
        let (rx, spy) = SpyMetricSink::new();
        let queuing = QueuingMetricSink::from(spy);

        queuing.emit("foo.counter:1|c").unwrap();
        queuing.emit("bar.counter:2|c").unwrap();
        queuing.emit("baz.counter:3|c").unwrap();
        queuing.worker.stop_and_wait();

        let m1 = rx.try_recv().unwrap();
        let m2 = rx.try_recv().unwrap();
        let m3 = rx.try_recv().unwrap();

        assert_eq!("foo.counter:1|c".as_bytes(), m1.as_slice());
        assert_eq!("bar.counter:2|c".as_bytes(), m2.as_slice());
        assert_eq!("baz.counter:3|c".as_bytes(), m3.as_slice());
    }

    #[test]
    fn test_queuing_sink_emit_panics() {
        let queuing = QueuingMetricSink::from(PanickingMetricSink::always());

        queuing.emit("foo.counter:4|c").unwrap();
        queuing.emit("foo.counter:5|c").unwrap();
        queuing.emit("foo.timer:34|ms").unwrap();
        queuing.worker.stop_and_wait();

        assert_eq!(3, queuing.panics());
    }

    // Make sure that subsequent metrics make it to the wrapped sink even when
    // the wrapped sink panics. This ensures that the thread running the sink
    // is restarted correctly and the worker and channel are in the correct state.
    #[test]
    fn test_queuing_sink_emit_recover_from_panics() {
        let queuing = QueuingMetricSink::from(PanickingMetricSink::every(2));

        queuing.emit("foo.counter:4|c").unwrap();
        queuing.emit("foo.counter:5|c").unwrap();
        queuing.emit("foo.timer:34|ms").unwrap();
        queuing.worker.stop_and_wait();

        assert_eq!(1, queuing.panics());
        assert_eq!(3, queuing.drained());
    }

    // Make sure that our queuing sink is unwind safe (it has the auto trait) and
    // that it handles any expected panics on its own, resulting in calling code not
    // seeing any panics.
    #[test]
    fn test_queuing_sink_panic_handler() {
        let queuing = QueuingMetricSink::from(PanickingMetricSink::always());

        let res = panic::catch_unwind(move || {
            queuing.emit("foo.counter:4|c").unwrap();
            queuing.emit("foo.counter:5|c").unwrap();
            queuing.emit("foo.timer:34|ms").unwrap();
            queuing.worker.stop_and_wait();
        });

        assert!(res.is_ok());
    }

    // Make sure that attempts to write to the sink start to fail when none of the
    // metrics have been drained by the wrapped sink. This is simulated by wrapping a
    // sink that sleeps indefinitely. Since all threads in Rust are daemon threads we
    // don't care that this thread won't stop, it'll be killed when the test process
    // exits.
    #[test]
    fn test_queuing_metric_sink_blocking_sink_back_pressure() {
        struct BlockingMetricSink;

        impl MetricSink for BlockingMetricSink {
            fn emit(&self, _m: &str) -> io::Result<usize> {
                loop {
                    thread::park();
                }
            }
        }

        let queueing = QueuingMetricSink::with_capacity(BlockingMetricSink, 1);
        let results = [
            queueing.emit("foo.counter:1|c"),
            queueing.emit("foo.counter:2|c"),
            queueing.emit("foo.counter:3|c"),
        ];

        let success = results.iter().map(|r| r.is_ok()).filter(|r| *r).count();
        let failure = results.iter().map(|r| r.is_err()).filter(|r| *r).count();

        // We've submitted three metrics to a queuing sink with a max capacity of one: at least
        // one of the submissions should be successfully queued and one of them should have failed.
        // Depending on how quickly the network thread of the sink started running, an entry may
        // have been removed from the queue (meaning that two submissions would have succeeded).
        assert!(
            success >= 1,
            "At least one submission to the queue should have succeeded"
        );
        assert!(failure >= 1, "At least one submission to the queue should have failed");
    }
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2019 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::io;
use std::panic::RefUnwindSafe;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use std::sync::Arc;
use std::thread;

use crossbeam_channel::{self, Receiver, Sender};

use crate::sinks::core::MetricSink;

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
#[derive(Debug, Clone)]
pub struct QueuingMetricSink {
    context: Arc<WorkerContext<String>>,
}

impl QueuingMetricSink {
    /// Construct a new `QueuingMetricSink` instance wrapping another sink
    /// implementation.
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
    pub fn from<T>(sink: T) -> QueuingMetricSink
    where
        T: MetricSink + Sync + Send + RefUnwindSafe + 'static,
    {
        let worker = Worker::new(move |v: String| {
            let _r = sink.emit(&v);
        });
        let context = Arc::new(WorkerContext::new(worker));
        spawn_worker_in_thread(Arc::clone(&context));

        QueuingMetricSink { context }
    }

    /// Return the number of times the wrapped sink or underlying worker thread
    /// has panicked and needed to be restarted. In typical use this should always
    /// be `0` but sometimes bugs happen.
    pub fn panics(&self) -> usize {
        self.context.stats.panics()
    }
}

impl MetricSink for QueuingMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.context.worker.submit(metric.to_string());
        Ok(metric.len())
    }
}

impl Drop for QueuingMetricSink {
    /// Send the worker a signal to stop processing metrics.
    ///
    /// Note that this destructor only sends the worker thread a signal to
    /// stop, it doesn't wait for it to stop.
    fn drop(&mut self) {
        self.context.worker.stop();
    }
}

/// Statistics about the worker running.
///
/// These statistics are only used for unit testing to verify that our
/// sentinel can handle thread panics and restart the thread the worker
/// is running in.
#[derive(Debug)]
struct WorkerStats {
    panics: AtomicUsize,
}

impl WorkerStats {
    fn new() -> WorkerStats {
        WorkerStats {
            panics: AtomicUsize::new(0),
        }
    }

    fn incr_panic(&self) {
        self.panics.fetch_add(1, Ordering::Release);
    }

    fn panics(&self) -> usize {
        self.panics.load(Ordering::Acquire)
    }
}

/// Holder for a worker and statistics about it.
///
/// Users of the context are expected to directly reference the members
/// of this struct.
#[derive(Debug)]
struct WorkerContext<T>
where
    T: Send + 'static,
{
    worker: Worker<T>,
    stats: WorkerStats,
}

impl<T> WorkerContext<T>
where
    T: Send + 'static,
{
    fn new(worker: Worker<T>) -> WorkerContext<T> {
        WorkerContext {
            worker,
            stats: WorkerStats::new(),
        }
    }
}

/// Create a thread and run the worker in it to completion
///
/// This function uses a `Sentinel` struct to make sure that any panics from
/// running the worker result in another thread being spawned to start running
/// the worker again.
fn spawn_worker_in_thread<T>(context: Arc<WorkerContext<T>>) -> thread::JoinHandle<()>
where
    T: Send + 'static,
{
    thread::spawn(move || {
        let mut sentinel = Sentinel::new(&context);
        context.worker.run();
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
struct Sentinel<'a, T>
where
    T: Send + 'static,
{
    context: &'a Arc<WorkerContext<T>>,
    active: bool,
}

impl<'a, T> Sentinel<'a, T>
where
    T: Send + 'static,
{
    fn new(context: &'a Arc<WorkerContext<T>>) -> Sentinel<'a, T> {
        Sentinel {
            context,
            active: true,
        }
    }

    fn cancel(&mut self) {
        self.active = false;
    }
}

impl<'a, T> Drop for Sentinel<'a, T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        if self.active {
            // This sentinel didn't have its `.cancel()`method called so
            // the thread must have panicked. Increment a counter indicating
            // that this was a panic and spawn a new thread with an Arc of
            // the worker and its context.
            self.context.stats.incr_panic();
            spawn_worker_in_thread(Arc::clone(self.context));
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
struct Worker<T>
where
    T: Send + 'static,
{
    task: Box<dyn Fn(T) -> () + Sync + Send + RefUnwindSafe + 'static>,
    sender: Sender<Option<T>>,
    receiver: Receiver<Option<T>>,
    stopped: AtomicBool,
}

impl<T> Worker<T>
where
    T: Send + 'static,
{
    fn new<F>(task: F) -> Worker<T>
    where
        F: Fn(T) -> () + Sync + Send + RefUnwindSafe + 'static,
    {
        let (tx, rx) = crossbeam_channel::unbounded();

        Worker {
            task: Box::new(task),
            sender: tx,
            receiver: rx,
            stopped: AtomicBool::new(false),
        }
    }

    fn submit(&self, v: T) {
        // Errors are ignored since the channel cannot be full and
        // disconnection means senders and receivers have been dropped
        // (which means we're shutting down and nothing could be sending
        // anything else via this channel anyway).
        let _ = self.sender.try_send(Some(v));
    }

    fn run(&self) {
        for opt in self.receiver.iter() {
            if let Some(v) = opt {
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

impl<T> fmt::Debug for Worker<T>
where
    T: Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Worker {{ ... }}")
    }
}

#[cfg(test)]
mod tests {
    use super::{QueuingMetricSink, Worker};
    use crate::sinks::core::MetricSink;
    use std::io;
    use std::panic;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_worker_submit_processes_event() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_ref = flag.clone();

        let task = move |v: String| {
            if v == "foo" {
                flag_ref.store(true, Ordering::Release);
            }
        };

        let worker = Arc::new(Worker::new(task));
        let worker_ref = worker.clone();

        let t = thread::spawn(move || {
            worker_ref.run();
        });

        worker.submit("bar".to_string());
        worker.submit("foo".to_string());
        worker.stop();
        t.join().unwrap();

        assert!(flag.load(Ordering::Acquire));
    }

    #[test]
    fn test_worker_stop() {
        let worker = Arc::new(Worker::new(move |_: String| {}));
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
        let worker = Arc::new(Worker::new(move |_: String| {}));
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
        let worker = Arc::new(Worker::new(move |_: String| {}));
        let worker_ref1 = worker.clone();
        let worker_ref2 = worker.clone();

        #[allow(unreachable_code)]
        let t1 = thread::spawn(move || {
            worker_ref1.submit(panic!("This thread is supposed to panic"));
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
        let worker = Arc::new(Worker::new(move |_: String| { panic!("This thread is supposed to panic"); }));
        let worker_ref1 = worker.clone();
        let worker_ref2 = worker.clone();

        let t1 = thread::spawn(move || {
            worker_ref1.submit("foo".to_owned());
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
        struct TestMetricSink {
            metrics: Arc<Mutex<Vec<String>>>,
        }

        impl TestMetricSink {
            fn new(metrics: Arc<Mutex<Vec<String>>>) -> TestMetricSink {
                TestMetricSink { metrics }
            }
        }

        impl MetricSink for TestMetricSink {
            fn emit(&self, m: &str) -> io::Result<usize> {
                let mut store = self.metrics.lock().unwrap();
                store.push(m.to_string());
                Ok(m.len())
            }
        }

        let store = Arc::new(Mutex::new(vec![]));
        let wrapped = TestMetricSink::new(store.clone());
        let queuing = QueuingMetricSink::from(wrapped);

        queuing.emit("foo.counter:1|c").unwrap();
        queuing.emit("bar.counter:2|c").unwrap();
        queuing.emit("baz.counter:3|c").unwrap();
        queuing.context.worker.stop_and_wait();

        assert_eq!("foo.counter:1|c".to_string(), store.lock().unwrap()[0]);
        assert_eq!("bar.counter:2|c".to_string(), store.lock().unwrap()[1]);
        assert_eq!("baz.counter:3|c".to_string(), store.lock().unwrap()[2]);
    }

    #[test]
    fn test_queuing_sink_emit_panics() {
        struct PanickingMetricSink;

        impl MetricSink for PanickingMetricSink {
            fn emit(&self, _m: &str) -> io::Result<usize> {
                panic!("This thread is supposed to panic");
            }
        }

        let queuing = QueuingMetricSink::from(PanickingMetricSink);
        queuing.emit("foo.counter:4|c").unwrap();
        queuing.emit("foo.counter:5|c").unwrap();
        queuing.emit("foo.timer:34|ms").unwrap();
        queuing.context.worker.stop_and_wait();

        assert_eq!(3, queuing.panics());
    }

    // Make sure that subsequent metrics make it to the wrapped sink even when
    // the wrapped sink panics. This ensures that the thread running the sink
    // is restarted correctly and the worker and channel are in the correct state.
    #[test]
    fn test_queuing_sink_emit_recover_from_panics() {
        struct SometimesPanickingMetricSink {
            metrics: Arc<Mutex<Vec<String>>>,
            counter: AtomicUsize,
        }

        impl SometimesPanickingMetricSink {
            fn new(metrics: Arc<Mutex<Vec<String>>>) -> Self {
                SometimesPanickingMetricSink {
                    metrics,
                    counter: AtomicUsize::new(0),
                }
            }
        }

        impl MetricSink for SometimesPanickingMetricSink {
            fn emit(&self, m: &str) -> io::Result<usize> {
                let val = self.counter.fetch_add(1, Ordering::Acquire);
                if val == 0 {
                    panic!("This thread is supposed to panic");
                }

                let mut store = self.metrics.lock().unwrap();
                store.push(m.to_string());
                Ok(m.len())
            }
        }

        let store = Arc::new(Mutex::new(vec![]));
        let queuing = QueuingMetricSink::from(SometimesPanickingMetricSink::new(store.clone()));

        queuing.emit("foo.counter:4|c").unwrap();
        queuing.emit("foo.counter:5|c").unwrap();
        queuing.emit("foo.timer:34|ms").unwrap();
        queuing.context.worker.stop_and_wait();

        assert_eq!(1, queuing.panics());
        assert_eq!("foo.counter:5|c".to_string(), store.lock().unwrap()[0]);
        assert_eq!("foo.timer:34|ms".to_string(), store.lock().unwrap()[1]);
    }

    // Make sure that our queuing sink is unwind safe (it has the auto trait) and
    // that it handles any expected panics on its own, resulting in calling code not
    // seeing any panics.
    #[test]
    fn test_queuing_sink_panic_handler() {
        struct PanickingMetricSink;

        impl MetricSink for PanickingMetricSink {
            fn emit(&self, _m: &str) -> io::Result<usize> {
                panic!("This thread is supposed to panic");
            }
        }

        let queuing = QueuingMetricSink::from(PanickingMetricSink);
        let res = panic::catch_unwind(move || {
            queuing.emit("foo.counter:4|c").unwrap();
            queuing.emit("foo.counter:5|c").unwrap();
            queuing.emit("foo.timer:34|ms").unwrap();
            queuing.context.worker.stop_and_wait();
        });

        assert!(res.is_ok());
    }
}

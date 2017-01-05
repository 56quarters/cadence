// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::io;
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crossbeam::sync::MsQueue;

use ::sinks::MetricSink;


/// Implementation of a `MetricSink` that wraps another implementation
/// and uses it to emit metrics asynchronously, in another thread.
///
/// Metics submitted to this sink are queued and sent to the wrapped sink
/// that is running in a separate thread. The wrapped implementation can
/// be any thread safe (`Sync` + `Send`) `MetricSink`. Results from the
/// wrapped implementation will be discarded.
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
///
/// **WARNING** This `MetricSink` is unstable and may change in a future
/// release. It's possible that it contains bugs. You are advised against
/// running it in production.
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
    /// sink should be thread safe (`Send + Sync`).
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
        where T: MetricSink + Sync + Send + 'static
    {
        let worker = Worker::new(move |v: String| { let _r = sink.emit(&v); });
        let context = Arc::new(WorkerContext::new(worker));
        spawn_worker_in_thread(context.clone());

        QueuingMetricSink { context: context }
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
        WorkerStats { panics: AtomicUsize::new(0) }
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
struct WorkerContext<T> where T: Send + 'static {
    worker: Worker<T>,
    stats: WorkerStats,
}


impl<T> WorkerContext<T> where T: Send + 'static {
    fn new(worker: Worker<T>) -> WorkerContext<T> {
        WorkerContext {
            worker: worker,
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
    where T: Send + 'static
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
struct Sentinel<'a, T> where T: Send + 'static {
    context: &'a Arc<WorkerContext<T>>,
    active: bool,
}


impl<'a, T> Sentinel<'a, T> where T: Send + 'static {
    fn new(context: &'a Arc<WorkerContext<T>>) -> Sentinel<'a, T> {
        Sentinel { context: context, active: true }
    }

    fn cancel(&mut self) {
        self.active = false;
    }
}


impl<'a, T> Drop for Sentinel<'a, T> where T: Send + 'static {
    fn drop(&mut self) {
        if self.active {
            // This sentinel didn't have its `.cancel()`method called so
            // the thread must have panicked. Increment a counter indicating
            // that this was a panic and spawn a new thread with an Arc of
            // the worker and its context.
            self.context.stats.incr_panic();
            spawn_worker_in_thread(self.context.clone());
        }
    }
}


/// Worker to repeatedly run a method consuming entries in a queue.
///
/// The `.run()` method of the worker is intended to be in a separate
/// thread (thread B). Meanwhile, the `.submit()`, `.stop()`,
/// `.stop_and_wait()`, and `.is_stopped()` methods are meant to be called
/// from the main thread (thread A).
///
/// This worker is stopped by recieving a "poison pill" message on the
/// queue that it is consuming messages from. Thus, calls to `.submit()`,
/// consuming messages in '.run()`, and `.stop()` typically involve no
/// locking.
///
/// However, in order to enable easier testing, after it stops receiving
/// messages the `.run()` method will use a `Mutex` to set a "stopped" flag
/// to allow callers waiting on a conditional variable (callers using
/// `.stop_and_wait()`) to wake up after the worker finally stops.
///
/// If you're just trying to make use of this worker you don't need to
/// worry about this, just call `.submit()`, `.run()`, and `.stop()`.
/// But, if you're wondering why this is mixing lock-free data structures
/// with locking and is genernally more complicated that it seems like
/// it should be: testing is the reason.
struct Worker<T> where T: Send + 'static {
    task: Box<Fn(T) -> () + Sync +  Send + 'static>,
    queue: MsQueue<Option<T>>,
    stopped: Mutex<bool>,
    cond: Condvar,
}


impl<T> Worker<T> where T: Send + 'static {
    fn new<F>(task: F) -> Worker<T> where F: Fn(T) -> () + Sync + Send + 'static {
        Worker {
            task: Box::new(task),
            queue: MsQueue::new(),
            stopped: Mutex::new(false),
            cond: Condvar::new(),
        }
    }

    fn submit(&self, v: T) {
        self.queue.push(Some(v));
    }

    fn run(&self) {
        loop {
            if let Some(v) = self.queue.pop() {
                (self.task)(v);
            } else {
                break;
            }
        }

        // Set the "stopped" flag so that callers using the `stop_and_wait`
        // method will wake up and see that we've stopped processing entries
        // in the queue. This is only for the benefit of unit testing.
        let mut stopped = self.stopped.lock().unwrap();
        *stopped = true;
        self.cond.notify_all();
    }

    fn stop(&self) {
        self.queue.push(None);
    }

    #[allow(dead_code)]
    fn stop_and_wait(&self) {
        let mut stopped = self.stopped.lock().unwrap();
        self.stop();

        while !*stopped {
            stopped = self.cond.wait(stopped).unwrap();
        }
    }

    #[allow(dead_code)]
    fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap()
    }
}


impl<T> fmt::Debug for Worker<T> where T: Send + 'static {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Worker {{ ... }}")
    }
}


#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use ::sinks::MetricSink;
    use super::{QueuingMetricSink, Worker};

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

    struct TestMetricSink {
        metrics: Arc<Mutex<Vec<String>>>,
    }

    impl TestMetricSink {
        fn new(store: Arc<Mutex<Vec<String>>>) -> TestMetricSink {
            TestMetricSink {
                metrics: store,
            }
        }
    }

    impl MetricSink for TestMetricSink {
        fn emit(&self, m: &str) -> io::Result<usize> {
            let mut store = self.metrics.lock().unwrap();
            store.push(m.to_string());
            Ok(m.len())
        }
    }

    #[test]
    fn test_queuing_sink_emit() {
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

    struct PanickingMetricSink;

    impl MetricSink for PanickingMetricSink {
        #[allow(unused_variables)]
        fn emit(&self, metric: &str) -> io::Result<usize> {
            panic!("This thread is supposed to panic, relax :p");
        }
    }

    #[test]
    fn test_queuing_sink_emit_panics() {
        let queuing = QueuingMetricSink::from(PanickingMetricSink);
        queuing.emit("foo.counter:4|c").unwrap();
        queuing.emit("foo.counter:5|c").unwrap();
        queuing.emit("foo.timer:34|ms").unwrap();
        queuing.context.worker.stop_and_wait();

        assert_eq!(3, queuing.panics());
    }
}

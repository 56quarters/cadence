// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2016 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::io;
use std::fmt;
use std::sync::Arc;
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
#[derive(Debug, Clone)]
pub struct QueuingMetricSink {
    worker: Arc<Worker<String>>,
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
        // In normal use we don't care about the thread that got started,
        // we just let the destructor give the worker a poison pill to stop
        // the `.run()` method being run in the thread.
        Self::from_sink_with_handle(sink).0
    }

    fn from_sink_with_handle<T>(sink: T) -> (QueuingMetricSink, thread::JoinHandle<()>)
        where T: MetricSink + Sync + Send + 'static
    {
        let worker = Arc::new(Worker::new(move |v: String| { let _r = sink.emit(&v); }));
        let worker_ref = worker.clone();

        // TODO: Implement a sentinal that uses Drop + thread::panicking() after each
        // job the worker handles to make sure that panic doesn't kill the thread and
        // prevent all metrics from getting sent.
        let handle = thread::spawn(move || {
            worker_ref.run();
        });

        // Return both the new sink and a handle to the thread that
        // we started to run the worker. In normal use we don't care
        // about the thread but it's useful to have a handle to make
        // sure that everything has been flushed to the wrapped sink
        // when running unit tests.
        (QueuingMetricSink { worker: worker }, handle)
    }
}


impl MetricSink for QueuingMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.worker.submit(metric.to_string());
        Ok(metric.len())
    }
}


impl Drop for QueuingMetricSink {
    fn drop(&mut self) {
        self.worker.stop();
    }
}


/// Worker to repeatedly run a method consuming entries in a queue.
///
/// The `.run()` method of the worker is intended to be in a separate
/// thread (thread B). Meanwhile, the `.submit()` and `.stop()` methods
/// are meant to be called from the main thread (thread A).
///
/// All pending entries are guaranteed to be processed before the worker
/// stops, even after the `.stop()` method has been called.
pub struct Worker<T> where T: Send + 'static {
    task: Box<Fn(T) -> () + Sync +  Send + 'static>,
    queue: MsQueue<Option<T>>,
}


impl<T> Worker<T> where T: Send + 'static {
    pub fn new<F>(task: F) -> Worker<T>
        where F: Fn(T) -> () + Sync + Send + 'static
    {
        Worker {
            task: Box::new(task),
            queue: MsQueue::new(),
        }
    }

    pub fn submit(&self, v: T) {
        self.queue.push(Some(v));
    }

    pub fn run(&self) {
        loop {
            if let Some(v) = self.queue.pop() {
                (self.task)(v);
            } else {
                return;
            }
        }
    }

    pub fn stop(&self) {
        self.queue.push(None);
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
    fn test_worker_stops() {
        let worker = Arc::new(Worker::new(move |_: String| {}));
        let worker_ref = worker.clone();

        let t = thread::spawn(move || {
            worker_ref.run();
        });

        worker.stop();
        t.join().unwrap();
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
        let handle = {
            let (queuing, handle) = QueuingMetricSink::from_sink_with_handle(wrapped);
            queuing.emit("foo.counter:1|c").unwrap();
            queuing.emit("bar.counter:2|c").unwrap();
            queuing.emit("baz.counter:3|c").unwrap();
            handle
        };

        handle.join().unwrap();
        assert_eq!("foo.counter:1|c".to_string(), store.lock().unwrap()[0]);
        assert_eq!("bar.counter:2|c".to_string(), store.lock().unwrap()[1]);
        assert_eq!("baz.counter:3|c".to_string(), store.lock().unwrap()[2]);
    }
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2019 Daniel Smith
// Copyright 2019-2020 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::io::Write;
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::io::MultiLineWriter;
use crate::sinks::core::MetricSink;

// Default size of the buffer for buffered metric sinks. This
// is a rather conservative value, picked for consistency with
// the UDP implementation.  Users may want to use a different
// value based on the configuration of the server their
// application is running on.
const DEFAULT_BUFFER_SIZE: usize = 512;

/// Implementation of a `MetricSink` that emits metrics over a Unix socket.
///
/// This is the most basic version of `MetricSink` that sends metrics over
/// a Unix socket. It accepts a Unix socket instance over which to write metrics
/// and the path of the socket for the Statsd server to send metrics to.
///
/// Each metric is sent to the Statsd server when the `.emit()` method is
/// called, in the thread of the caller.
///
/// Note that unlike the UDP sinks, if there is no receiving socket at the path
/// specified or nothing listening at the path, an error will be returned when
/// metrics are emitted.
#[derive(Debug)]
pub struct UnixMetricSink {
    socket: UnixDatagram,
    path: PathBuf,
}

impl UnixMetricSink {
    /// Construct a new `UnixMetricSink` instance.
    ///
    /// The socket does not need to be bound (i.e. `UnixDatagram::unbound()` is
    /// fine) but should have any desired configuration already applied
    /// (blocking vs non-blocking, timeouts, etc.).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixDatagram;
    /// use cadence::UnixMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// let sink = UnixMetricSink::from("/run/statsd.sock", socket);
    /// ```
    ///
    /// To send metrics over a non-blocking socket, simply put the socket
    /// in non-blocking mode before creating the Unix metric sink.
    ///
    /// # Non-blocking Example
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixDatagram;
    /// use cadence::UnixMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// socket.set_nonblocking(true).unwrap();
    /// let sink = UnixMetricSink::from("/run/statsd.sock", socket);
    /// ```
    pub fn from<P>(path: P, socket: UnixDatagram) -> UnixMetricSink
    where
        P: AsRef<Path>,
    {
        UnixMetricSink {
            path: path.as_ref().to_path_buf(),
            socket,
        }
    }
}

impl MetricSink for UnixMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.socket.send_to(metric.as_bytes(), self.path.as_path())
    }
}

/// Adapter for writing to a `UnixDatagram` socket via the `Write` trait
#[derive(Debug)]
pub(crate) struct UnixWriteAdapter {
    path: PathBuf,
    socket: UnixDatagram,
}

impl UnixWriteAdapter {
    fn new<P>(socket: UnixDatagram, path: P) -> UnixWriteAdapter
    where
        P: AsRef<Path>,
    {
        UnixWriteAdapter {
            path: path.as_ref().to_path_buf(),
            socket,
        }
    }
}

impl Write for UnixWriteAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send_to(buf, &self.path)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Implementation of a `MetricSink` that buffers metrics before
/// sending them to a Unix socket.
///
/// Metrics are line buffered, meaning that a trailing "\n" is added
/// after each metric written to this sink. When the buffer is sufficiently
/// full and a write is attempted, the contents of the buffer are flushed to
/// a Unix socket and then the metric is written to the buffer. The buffer is
/// also flushed when this sink is destroyed.
///
/// The default size of the buffer is 512 bytes. This is to be consistent with
/// the default for the `BufferedUdpMetricSink`. The buffer size can be customized
/// using the `with_capacity` method to create the sink if desired.
///
/// If a metric larger than the buffer is emitted, it will be written
/// directly to the underlying Unix socket, bypassing the buffer.
///
/// Note that since metrics are buffered until a certain size is reached, it's
/// possible that they may sit in the buffer for a while for applications
/// that do not emit metrics frequently or at a high volume. For these low-
/// throughput use cases, it may make more sense to use the `UnixMetricSink`
/// since it sends metrics immediately with no buffering.
///
/// Also note that unlike the UDP sinks, if there is no receiving socket at the path
/// specified or nothing listening at the path, an error will be returned when
/// metrics are emitted (though this may not happen on every write due to buffering).
#[derive(Debug)]
pub struct BufferedUnixMetricSink {
    buffer: Mutex<MultiLineWriter<UnixWriteAdapter>>,
}

impl BufferedUnixMetricSink {
    /// Construct a new `BufferedUnixMetricSink` instance with a default
    /// buffer size of 512 bytes.
    ///
    /// The socket does not need to be bound (i.e. `UnixDatagram::unbound()` is
    /// fine) but should have any desired configuration already applied
    /// (blocking vs non-blocking, timeouts, etc.).
    ///
    /// Writes to this sink are automatically suffixed with a Unix newline
    /// ('\n') by the sink and stored in a 512 byte buffer until the buffer
    /// is full or this sink is destroyed, at which point the buffer will be
    /// flushed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixDatagram;
    /// use cadence::BufferedUnixMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// let sink = BufferedUnixMetricSink::from("/run/statsd.sock", socket);
    /// ```
    pub fn from<P>(path: P, socket: UnixDatagram) -> BufferedUnixMetricSink
    where
        P: AsRef<Path>,
    {
        Self::with_capacity(path, socket, DEFAULT_BUFFER_SIZE)
    }

    /// Construct a new `BufferedUnixMetricSink` instance with a custom
    /// buffer size.
    ///
    /// The socket does not need to be bound (i.e. `UnixDatagram::unbound()` is
    /// fine) but should have with any desired configuration already applied
    /// (blocking vs non-blocking, timeouts, etc.).
    ///
    /// Writes to this sink are automatically suffixed  with a Unix newline
    /// ('\n') by the sink and stored in a buffer until the buffer is full
    /// or this sink is destroyed, at which point the buffer will be flushed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixDatagram;
    /// use cadence::BufferedUnixMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// let sink = BufferedUnixMetricSink::with_capacity("/run/statsd.sock", socket, 1432);
    /// ```
    pub fn with_capacity<P>(path: P, socket: UnixDatagram, cap: usize) -> BufferedUnixMetricSink
    where
        P: AsRef<Path>,
    {
        BufferedUnixMetricSink {
            buffer: Mutex::new(MultiLineWriter::new(
                cap,
                UnixWriteAdapter::new(socket, path),
            )),
        }
    }
}

impl MetricSink for BufferedUnixMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        let mut writer = self.buffer.lock().unwrap();
        writer.write(metric.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::{BufferedUnixMetricSink, MetricSink, UnixMetricSink};
    use crate::test::UnixServerHarness;
    use std::os::unix::net::UnixDatagram;

    #[test]
    fn test_unix_metric_sink() {
        let harness = UnixServerHarness::new("test_unix_metric_sink");

        harness.run_quiet(|path| {
            let socket = UnixDatagram::unbound().unwrap();
            let sink = UnixMetricSink::from(path, socket);

            assert_eq!(7, sink.emit("buz:1|m").unwrap());
        });
    }

    #[test]
    fn test_non_blocking_unix_metric_sink() {
        let harness = UnixServerHarness::new("test_non_blocking_unix_metric_sink");

        harness.run_quiet(|path| {
            let socket = UnixDatagram::unbound().unwrap();
            socket.set_nonblocking(true).unwrap();
            let sink = UnixMetricSink::from(path, socket);

            assert_eq!(7, sink.emit("baz:1|m").unwrap());
        });
    }

    #[test]
    fn test_buffered_unix_metric_sink() {
        let harness = UnixServerHarness::new("test_buffered_unix_metric_sink");

        harness.run_quiet(|path| {
            let socket = UnixDatagram::unbound().unwrap();

            // Set the capacity of the buffer such that we know it will
            // be flushed as a response to the metrics we're writing.
            let sink = BufferedUnixMetricSink::with_capacity(path, socket, 16);

            // Note that we're including an extra byte in the expected
            // number written since each metric is followed by a '\n' at
            // the end.
            assert_eq!(9, sink.emit("foo:54|c").unwrap());
            assert_eq!(9, sink.emit("foo:67|c").unwrap());
        });
    }
}

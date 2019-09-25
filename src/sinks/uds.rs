// Cadence - An extensible Statsd client for Rust!
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

/// Implementation of a `MetricSink` that emits metrics over UDS.
///
/// This is the most basic version of `MetricSink` that sends metrics over
/// UDS. It accepts a UDS socket instance over which to write metrics and
/// the address of the Statsd server to send packets to.
///
/// Each metric is sent to the Statsd server when the `.emit()` method is
/// called, in the thread of the caller.
#[derive(Debug)]
pub struct UdsMetricSink {
    socket: UnixDatagram,
    path: PathBuf,
}

impl UdsMetricSink {
    /// Construct a new `UdsMetricSink` instance.
    ///
    /// The stream should already be bound to a UDS with any desired
    /// configuration applied (blocking vs non-blocking, timeouts, etc.).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixDatagram;
    /// use cadence::UdsMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// let sink = UdsMetricSink::from(socket, "/tmp/sock");
    /// ```
    ///
    /// To send metrics over a non-blocking socket, simply put the socket
    /// in non-blocking mode before creating the UDS metric sink.
    ///
    /// # Non-blocking Example
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixDatagram;
    /// use cadence::UdsMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// socket.set_nonblocking(true).unwrap();
    /// let sink = UdsMetricSink::from(socket, "/tmp/sock");
    /// ```
    pub fn from<P: AsRef<Path>>(socket: UnixDatagram, path: P) -> UdsMetricSink {
        UdsMetricSink {
            socket: socket,
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl MetricSink for UdsMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.socket.send_to(metric.as_bytes(), self.path.as_path())
    }
}

/// Adapter for writing to a `UnixDatagram` socket via the `Write` trait
#[derive(Debug)]
pub(crate) struct UdsWriteAdapter {
    socket: UnixDatagram,
    path: PathBuf,
}

impl UdsWriteAdapter {
    fn new<P: AsRef<Path>>(socket: UnixDatagram, path: P) -> UdsWriteAdapter {
        UdsWriteAdapter {
            socket: socket,
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl Write for UdsWriteAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send_to(buf, self.path.as_path())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Implementation of a `MetricSink` that buffers metrics before
/// sending them to a UDS socket.
///
/// Metrics are line buffered, meaning that a trailing "\n" is added
/// after each metric written to this sink. When the buffer is sufficiently
/// full and a write is attempted, the contents of the buffer are flushed to
/// a UDS socket and then the metric is written to the buffer. The buffer is
/// also flushed when this sink is destroyed.
///
/// The default size of the buffer is 512 bytes. This is to be consistent with
/// the default for the BufferedUdpMetricSink. The buffer size can be customized
/// using the `with_capacity` method to create the sink if desired.
///
/// If a metric larger than the buffer is emitted, it will be written
/// directly to the underlying UDS socket, bypassing the buffer.
#[derive(Debug)]
pub struct BufferedUdsMetricSink {
    buffer: Mutex<MultiLineWriter<UdsWriteAdapter>>,
}

impl BufferedUdsMetricSink {
    /// Construct a new `BufferedUdsMetricSink` instance with a default
    /// buffer size of 512 bytes.
    ///
    /// The socket should already be bound to a local address with any desired
    /// configuration applied (blocking vs non-blocking, timeouts, etc.).
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
    /// use cadence::BufferedUdsMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// let sink = BufferedUdsMetricSink::from(socket, "/tmp/sock");
    /// ```
    pub fn from<P: AsRef<Path>>(socket: UnixDatagram, path: P) -> BufferedUdsMetricSink {
        Self::with_capacity(socket, path, DEFAULT_BUFFER_SIZE)
    }

    /// Construct a new `BufferedUdsMetricSink` instance with a custom
    /// buffer size.
    ///
    /// The socket should already be bound to a local address with any desired
    /// configuration applied (blocking vs non-blocking, timeouts, etc.).
    ///
    /// Writes to this sink are automatically suffixed  with a Unix newline
    /// ('\n') by the sink and stored in a buffer until the buffer is full
    /// or this sink is destroyed, at which point the buffer will be flushed.
    ///
    /// For guidance on sizing your buffer see the
    /// [Statsd docs](https://github.com/etsy/statsd/blob/master/docs/metric_types.md#multi-metric-packets).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::os::unix::net::UnixDatagram;
    /// use cadence::BufferedUdsMetricSink;
    ///
    /// let socket = UnixDatagram::unbound().unwrap();
    /// let sink = BufferedUdsMetricSink::with_capacity(socket, "/tmp/sock", 1432);
    /// ```
    pub fn with_capacity<P: AsRef<Path>>(
        socket: UnixDatagram,
        path: P,
        cap: usize,
    ) -> BufferedUdsMetricSink {
        BufferedUdsMetricSink {
            buffer: Mutex::new(MultiLineWriter::new(
                cap,
                UdsWriteAdapter::new(socket, path),
            )),
        }
    }
}

impl MetricSink for BufferedUdsMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        let mut writer = self.buffer.lock().unwrap();
        writer.write(metric.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::{BufferedUdsMetricSink, MetricSink, UdsMetricSink};
    use std::os::unix::net::UnixDatagram;
    use tempdir::TempDir;

    #[test]
    fn test_uds_metric_sink() {
        let socket = UnixDatagram::unbound().unwrap();
        let tmp_dir = TempDir::new("testing").unwrap();
        let file_path = tmp_dir.path().join("tmp.sock");
        // Create a listener.
        let _listener = UnixDatagram::bind(&file_path);
        let sink = UdsMetricSink::from(socket, file_path);
        assert_eq!(7, sink.emit("buz:1|m").unwrap());
    }

    #[test]
    fn test_non_blocking_uds_metric_sink() {
        let socket = UnixDatagram::unbound().unwrap();
        socket.set_nonblocking(true).unwrap();
        let tmp_dir = TempDir::new("testing").unwrap();
        let file_path = tmp_dir.path().join("tmp.sock");
        // Create a listener.
        let _listener = UnixDatagram::bind(&file_path);
        let sink = UdsMetricSink::from(socket, file_path);
        assert_eq!(7, sink.emit("baz:1|m").unwrap());
    }

    #[test]
    fn test_buffered_uds_metric_sink() {
        let socket = UnixDatagram::unbound().unwrap();
        let tmp_dir = TempDir::new("testing").unwrap();
        let file_path = tmp_dir.path().join("tmp.sock");
        // Create a listener.
        let _listener = UnixDatagram::bind(&file_path);
        // Set the capacity of the buffer such that we know it will
        // be flushed as a response to the metrics we're writing.
        let sink = BufferedUdsMetricSink::with_capacity(socket, file_path, 16);

        // Note that we're including an extra byte in the expected
        // number written since each metric is followed by a '\n' at
        // the end.
        assert_eq!(9, sink.emit("foo:54|c").unwrap());
        assert_eq!(9, sink.emit("foo:67|c").unwrap());
    }
}

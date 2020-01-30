// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2020 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Mutex;

use crate::io::MultiLineWriter;
use crate::sinks::core::MetricSink;
use crate::types::{ErrorKind, MetricError, MetricResult};

// Default size of the buffer for buffered metric sinks. This
// is a rather conservative value, picked to make sure the entire
// buffer fits in a small UDP packet. Users may want to use a
// different value based on the configuration of the network
// their application runs in.
const DEFAULT_BUFFER_SIZE: usize = 512;

/// Attempt to convert anything implementing the `ToSocketAddrs` trait
/// into a concrete `SocketAddr` instance, returning an `InvalidInput`
/// error if the address could not be parsed.
// Public portion of the API (the sink constructors) is pass by value so
// there's no point in changing this to be pass by reference yet.
#[allow(clippy::needless_pass_by_value)]
fn get_addr<A: ToSocketAddrs>(addr: A) -> MetricResult<SocketAddr> {
    match addr.to_socket_addrs()?.next() {
        Some(addr) => Ok(addr),
        None => Err(MetricError::from((
            ErrorKind::InvalidInput,
            "No socket addresses yielded",
        ))),
    }
}

/// Implementation of a `MetricSink` that emits metrics over UDP.
///
/// This is the most basic version of `MetricSink` that sends metrics over
/// UDP. It accepts a UDP socket instance over which to write metrics and
/// the address of the Statsd server to send packets to.
///
/// Each metric is sent to the Statsd server when the `.emit()` method is
/// called, in the thread of the caller.
#[derive(Debug)]
pub struct UdpMetricSink {
    addr: SocketAddr,
    socket: UdpSocket,
}

impl UdpMetricSink {
    /// Construct a new `UdpMetricSink` instance.
    ///
    /// The address should be the address of the remote metric server to
    /// emit metrics to over UDP. The socket should already be bound to a
    /// local address with any desired configuration applied (blocking vs
    /// non-blocking, timeouts, etc.).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{UdpMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let sink = UdpMetricSink::from(host, socket);
    /// ```
    ///
    /// To send metrics over a non-blocking socket, simply put the socket
    /// in non-blocking mode before creating the UDP metric sink.
    ///
    /// # Non-blocking Example
    ///
    /// Note that putting the UDP socket into non-blocking mode is the
    /// default when sink and socket are automatically created with the
    /// `StatsdClient::from_udp_host` method.
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{UdpMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// socket.set_nonblocking(true).unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let sink = UdpMetricSink::from(host, socket);
    /// ```
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed
    pub fn from<A>(to_addr: A, socket: UdpSocket) -> MetricResult<UdpMetricSink>
    where
        A: ToSocketAddrs,
    {
        let addr = get_addr(to_addr)?;
        Ok(UdpMetricSink { addr, socket })
    }
}

impl MetricSink for UdpMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.socket.send_to(metric.as_bytes(), &self.addr)
    }
}

/// Adapter for writing to a `UdpSocket` via the `Write` trait
#[derive(Debug)]
pub(crate) struct UdpWriteAdapter {
    addr: SocketAddr,
    socket: UdpSocket,
}

impl UdpWriteAdapter {
    pub(crate) fn new(addr: SocketAddr, socket: UdpSocket) -> UdpWriteAdapter {
        UdpWriteAdapter { addr, socket }
    }
}

impl Write for UdpWriteAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send_to(buf, &self.addr)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Implementation of a `MetricSink` that buffers metrics before
/// sending them to a UDP socket.
///
/// Metrics are line buffered, meaning that a trailing "\n" is added
/// after each metric written to this sink. When the buffer is sufficiently
/// full and a write is attempted, the contents of the buffer are flushed to
/// a UDP socket and then the metric is written to the buffer. The buffer is
/// also flushed when this sink is destroyed.
///
/// The default size of the buffer is 512 bytes. This is the "safest"
/// size for a UDP packet according to the Etsy Statsd docs. The
/// buffer size can be customized using the `with_capacity` method
/// to create the sink if desired.
///
/// If a metric larger than the buffer is emitted, it will be written
/// directly to the underlying UDP socket, bypassing the buffer.
///
/// Note that since metrics are buffered until a certain size is reached, it's
/// possible that they may sit in the buffer for a while for applications
/// that do not emit metrics frequently or at a high volume. For these low-
/// throughput use cases, it may make more sense to use the `UdpMetricSink`
/// since it sends metrics immediately with no buffering.
#[derive(Debug)]
pub struct BufferedUdpMetricSink {
    buffer: Mutex<MultiLineWriter<UdpWriteAdapter>>,
}

impl BufferedUdpMetricSink {
    /// Construct a new `BufferedUdpMetricSink` instance with a default
    /// buffer size of 512 bytes.
    ///
    /// The address should be the address of the remote metric server to
    /// emit metrics to over UDP. The socket should already be bound to a
    /// local address with any desired configuration applied (blocking vs
    /// non-blocking, timeouts, etc.).
    ///
    /// Writes to this sink are automatically suffixed with a Unix newline
    /// ('\n') by the sink and stored in a 512 byte buffer until the buffer
    /// is full or this sink is destroyed, at which point the buffer will be
    /// flushed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{BufferedUdpMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let sink = BufferedUdpMetricSink::from(host, socket);
    /// ```
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed
    pub fn from<A>(sink_addr: A, socket: UdpSocket) -> MetricResult<BufferedUdpMetricSink>
    where
        A: ToSocketAddrs,
    {
        Self::with_capacity(sink_addr, socket, DEFAULT_BUFFER_SIZE)
    }

    /// Construct a new `BufferedUdpMetricSink` instance with a custom
    /// buffer size.
    ///
    /// The address should be the address of the remote metric server to
    /// emit metrics to over UDP. The socket should already be bound to a
    /// local address with any desired configuration applied (blocking vs
    /// non-blocking, timeouts, etc.).
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
    /// use std::net::UdpSocket;
    /// use cadence::{BufferedUdpMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let sink = BufferedUdpMetricSink::with_capacity(host, socket, 1432);
    /// ```
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed
    pub fn with_capacity<A>(
        sink_addr: A,
        socket: UdpSocket,
        cap: usize,
    ) -> MetricResult<BufferedUdpMetricSink>
    where
        A: ToSocketAddrs,
    {
        let addr = get_addr(sink_addr)?;
        Ok(BufferedUdpMetricSink {
            buffer: Mutex::new(MultiLineWriter::new(
                cap,
                UdpWriteAdapter::new(addr, socket),
            )),
        })
    }
}

impl MetricSink for BufferedUdpMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        let mut writer = self.buffer.lock().unwrap();
        writer.write(metric.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::{get_addr, BufferedUdpMetricSink, MetricSink, UdpMetricSink};
    use std::net::UdpSocket;

    #[test]
    fn test_get_addr_bad_address() {
        let res = get_addr("asdf");
        assert!(res.is_err());
    }

    #[test]
    fn test_get_addr_valid_address() {
        let res = get_addr("127.0.0.1:8125");
        assert!(res.is_ok());
    }

    #[test]
    fn test_udp_metric_sink() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let sink = UdpMetricSink::from("127.0.0.1:8125", socket).unwrap();
        assert_eq!(7, sink.emit("buz:1|m").unwrap());
    }

    #[test]
    fn test_non_blocking_udp_metric_sink() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_nonblocking(true).unwrap();
        let sink = UdpMetricSink::from("127.0.0.1:8125", socket).unwrap();
        assert_eq!(7, sink.emit("baz:1|m").unwrap());
    }

    #[test]
    fn test_buffered_udp_metric_sink() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        // Set the capacity of the buffer such that we know it will
        // be flushed as a response to the metrics we're writing.
        let sink = BufferedUdpMetricSink::with_capacity("127.0.0.1:8125", socket, 16).unwrap();

        // Note that we're including an extra byte in the expected
        // number written since each metric is followed by a '\n' at
        // the end.
        assert_eq!(9, sink.emit("foo:54|c").unwrap());
        assert_eq!(9, sink.emit("foo:67|c").unwrap());
    }
}

// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::io::MultiLineWriter;
use crate::sinks::core::{MetricSink, SinkStats, SocketStats};
use crate::sinks::resolve::{PeriodicResolver, Resolver, StaticResolver};
use crate::types::MetricResult;
use std::io::{self, Write};
use std::net::{ToSocketAddrs, UdpSocket};
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fmt, thread};

// Default size of the buffer for buffered metric sinks. This
// is a rather conservative value, picked to make sure the entire
// buffer fits in a small UDP packet. Users may want to use a
// different value based on the configuration of the network
// their application runs in.
const DEFAULT_BUFFER_SIZE: usize = 512;

/// Create a new shared `Resolver` implementation for the given
/// address based on if a refresh has been set.
///
/// An error is returned if the initial resolution of `addr` fails.
fn get_resolver<A>(
    addr: A,
    period: Option<Duration>,
    error_handler: Option<Box<dyn Fn(io::Error) + Sync + Send + RefUnwindSafe>>,
) -> MetricResult<Arc<dyn Resolver + Send + Sync + RefUnwindSafe>>
where
    A: ToSocketAddrs + fmt::Debug + Send + Sync + RefUnwindSafe + 'static,
{
    match period {
        Some(duration) => {
            let error_handler = error_handler.unwrap_or_else(|| Box::new(|_e| {}));
            let sleep = |d| thread::sleep(d);
            let resolver = Arc::new(PeriodicResolver::new(addr, duration, error_handler, sleep)?);
            let resolver_c = resolver.clone();
            crate::sync::execute(move || resolver_c.run());
            Ok(resolver)
        }
        None => Ok(Arc::new(StaticResolver::new(addr)?)),
    }
}

/// Implementation of a builder pattern for `UdpMetricSink`.
///
/// The builder can be used to set how often the hostname to send
/// metrics to is re-resolved via DNS and how to handle errors when
/// re-resolving the hostname. If these options are _not_ set, the
/// default behavior is to only resolve the hostname for sending
/// metrics upon creation of the sink, the same as when the sink
/// is created using `UdpMetricSink::from()`.
///
/// When the option to re-resolve hostnames to send metrics to is
/// enabled, a new thread is spawned when the sink is created that
/// will run until the sink is dropped, updating the `SocketAddr`
/// used for sending metrics in the background.
///
/// # Example
///
/// ```no_run
/// use std::net::UdpSocket;
/// use std::time::Duration;
/// use cadence::{UdpMetricSink, DEFAULT_PORT};
///
/// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
/// let host = ("metrics.example.com", DEFAULT_PORT);
/// let sink = UdpMetricSink::builder()
///     .with_resolver_period(Duration::from_secs(5))
///     .with_resolver_error_handler(|e| {
///         eprintln!("failed to re-resolve address: {}", e);
///     })
///     .build(host, socket)
///     .unwrap();
/// ```
#[derive(Default)]
pub struct UdpMetricSinkBuilder {
    resolver_period: Option<Duration>,
    resolver_error_handler: Option<Box<dyn Fn(io::Error) + Sync + Send + RefUnwindSafe>>,
}

impl UdpMetricSinkBuilder {
    /// Construct a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a new `UdpMetricSink` instance that emits metrics
    /// using the provided address and socket.
    ///
    /// The `UdpMetricSink` may optionally re-resolve the provided
    /// address using DNS periodically. If provided, an error handler
    /// will be invoked when the address cannot be resolved. The
    /// error handler runs in the same thread as the resolver logic,
    /// different from the thread the sink runs in, and must not
    /// panic.
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed
    pub fn build<A>(self, to_addr: A, socket: UdpSocket) -> MetricResult<UdpMetricSink>
    where
        A: ToSocketAddrs + fmt::Debug + Send + Sync + RefUnwindSafe + 'static,
    {
        let resolver = get_resolver(to_addr, self.resolver_period, self.resolver_error_handler)?;
        let stats = SocketStats::default();
        Ok(UdpMetricSink {
            resolver,
            socket,
            stats,
        })
    }

    /// Set how often to re-resolve the metric server address for this sink.
    ///
    /// If not called, the metric server address is only resolved once when the
    /// sink is constructed.
    pub fn with_resolver_period(mut self, duration: Duration) -> Self {
        self.resolver_period = Some(duration);
        self
    }

    /// Set the error handler to use when re-resolving the metric server address
    /// fails.
    ///
    /// If `with_resolver_period` has not been called before the sink is constructed,
    /// the error handler is not used.
    pub fn with_resolver_error_handler<F>(mut self, error_handler: F) -> Self
    where
        F: Fn(io::Error) + Sync + Send + RefUnwindSafe + 'static,
    {
        self.resolver_error_handler = Some(Box::new(error_handler));
        self
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
pub struct UdpMetricSink {
    resolver: Arc<dyn Resolver + Send + Sync + RefUnwindSafe>,
    socket: UdpSocket,
    stats: SocketStats,
}

impl UdpMetricSink {
    /// Construct a new builder for `UdpMetricSink` that can be used to
    /// customize advanced behavior of the sink.
    ///
    /// See `UdpMetricSinkBuilder` for more information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use std::time::Duration;
    /// use cadence::{UdpMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let sink = UdpMetricSink::builder()
    ///     .with_resolver_period(Duration::from_secs(5))
    ///     .with_resolver_error_handler(|e| {
    ///         eprintln!("failed to re-resolve address: {}", e);
    ///     })
    ///     .build(host, socket)
    ///     .unwrap();
    /// ```
    pub fn builder() -> UdpMetricSinkBuilder {
        UdpMetricSinkBuilder::new()
    }

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
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use cadence::{UdpMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// socket.set_nonblocking(true).unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let sink = UdpMetricSink::from(host, socket).unwrap();
    /// ```
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed
    pub fn from<A>(to_addr: A, socket: UdpSocket) -> MetricResult<Self>
    where
        A: ToSocketAddrs,
    {
        // TODO: Explain the bounds of A
        let resolver = Arc::new(StaticResolver::new(to_addr)?);
        let stats = SocketStats::default();
        Ok(Self {
            resolver,
            socket,
            stats,
        })
    }
}

impl MetricSink for UdpMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.stats.update(
            self.socket.send_to(metric.as_bytes(), self.resolver.get_addr()),
            metric.len(),
        )
    }

    fn stats(&self) -> SinkStats {
        (&self.stats).into()
    }
}

impl Drop for UdpMetricSink {
    fn drop(&mut self) {
        self.resolver.stop();
    }
}

impl fmt::Debug for UdpMetricSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UdpMetricSink")
            .field("resolver", &"...")
            .field("socket", &self.socket)
            .field("stats", &self.stats)
            .finish()
    }
}

/// Adapter for writing to a `UdpSocket` via the `Write` trait
pub(crate) struct UdpWriteAdapter {
    resolver: Arc<dyn Resolver + Send + Sync + RefUnwindSafe>,
    socket: UdpSocket,
    stats: SocketStats,
}

impl UdpWriteAdapter {
    pub(crate) fn new(
        resolver: Arc<dyn Resolver + Send + Sync + RefUnwindSafe>,
        socket: UdpSocket,
        stats: SocketStats,
    ) -> Self {
        Self {
            resolver,
            socket,
            stats,
        }
    }
}

impl Write for UdpWriteAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stats
            .update(self.socket.send_to(buf, self.resolver.get_addr()), buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for UdpWriteAdapter {
    fn drop(&mut self) {
        self.resolver.stop();
    }
}

impl fmt::Debug for UdpWriteAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UdpWriteAdapter")
            .field("resolver", &"...")
            .field("socket", &self.socket)
            .field("stats", &self.stats)
            .finish()
    }
}

/// Implementation of a builder pattern for `BufferedUdpMetricSink`.
///
/// The builder can be used to set the capacity of the buffer, how
/// often the hostname to send metrics to is re-resolved via DNS, and
/// how to handle errors when re-resolving the hostname. If these
/// options are _not_ set, the default behavior is to only resolve
/// the hostname for sending metrics upon creation of the sink and
/// use a default buffer size, the same as when the sink is created
/// using `BufferedUdpMetricSink::from()`.
///
/// When the option to re-resolve hostnames to send metrics to is
/// enabled, a new thread is spawned when the sink is created that
/// will run until the sink is dropped, updating the `SocketAddr`
/// used for sending metrics in the background.
///
/// # Example
///
/// ```no_run
/// use std::net::UdpSocket;
/// use std::time::Duration;
/// use cadence::{BufferedUdpMetricSink, DEFAULT_PORT};
///
/// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
/// let host = ("metrics.example.com", DEFAULT_PORT);
/// let sink = BufferedUdpMetricSink::builder()
///     .with_capacity(1024)
///     .with_resolver_period(Duration::from_secs(5))
///     .with_resolver_error_handler(|e| {
///         eprintln!("failed to re-resolve address: {}", e);
///     })
///     .build(host, socket)
///     .unwrap();
/// ```
#[derive(Default)]
pub struct BufferedUdpMetricSinkBuilder {
    capacity: Option<usize>,
    resolver_period: Option<Duration>,
    resolver_error_handler: Option<Box<dyn Fn(io::Error) + Sync + Send + RefUnwindSafe>>,
}

impl BufferedUdpMetricSinkBuilder {
    /// Construct a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a new `BufferedUdpMetricSink` instance that emits metrics
    /// using the provided address and socket.
    ///
    /// The `BufferedUdpMetricSink` may optionally re-resolve the provided
    /// address using DNS periodically. If provided, an error handler
    /// will be invoked when the address cannot be resolved. The
    /// error handler runs in the same thread as the resolver logic,
    /// different from the thread the sink runs in, and must not
    /// panic.
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed
    pub fn build<A>(self, to_addr: A, socket: UdpSocket) -> MetricResult<BufferedUdpMetricSink>
    where
        A: ToSocketAddrs + fmt::Debug + Send + Sync + RefUnwindSafe + 'static,
    {
        let resolver = get_resolver(to_addr, self.resolver_period, self.resolver_error_handler)?;
        let stats = SocketStats::default();
        Ok(BufferedUdpMetricSink {
            buffer: Mutex::new(MultiLineWriter::new(
                UdpWriteAdapter::new(resolver, socket, stats.clone()),
                self.capacity.unwrap_or(DEFAULT_BUFFER_SIZE),
            )),
            stats,
        })
    }

    /// Set the capacity of the buffer used for metrics before sending
    /// them to underlying socket.
    ///
    /// If not called, the default buffer size (512 bytes) is used.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Set how often to re-resolve the metric server address for this sink.
    ///
    /// If not called, the metric server address is only resolved once when the
    /// sink is constructed.
    pub fn with_resolver_period(mut self, duration: Duration) -> Self {
        self.resolver_period = Some(duration);
        self
    }

    /// Set the error handler to use when re-resolving the metric server address
    /// fails.
    ///
    /// If `with_resolver_period` has not been called before the sink is constructed,
    /// the error handler is not used.
    pub fn with_resolver_error_handler<F>(mut self, error_handler: F) -> Self
    where
        F: Fn(io::Error) + Sync + Send + RefUnwindSafe + 'static,
    {
        self.resolver_error_handler = Some(Box::new(error_handler));
        self
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
    stats: SocketStats,
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
    pub fn from<A>(sink_addr: A, socket: UdpSocket) -> MetricResult<Self>
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
    pub fn with_capacity<A>(to_addr: A, socket: UdpSocket, cap: usize) -> MetricResult<Self>
    where
        A: ToSocketAddrs,
    {
        let resolver = Arc::new(StaticResolver::new(to_addr)?);
        let stats = SocketStats::default();
        Ok(BufferedUdpMetricSink {
            buffer: Mutex::new(MultiLineWriter::new(
                UdpWriteAdapter::new(resolver, socket, stats.clone()),
                cap,
            )),
            stats,
        })
    }

    /// Construct a new builder for `BufferedUdpMetricSink` that can be used to
    /// customize advanced behavior of the sink.
    ///
    /// See `BufferedUdpMetricSinkBuilder` for more information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::net::UdpSocket;
    /// use std::time::Duration;
    /// use cadence::{BufferedUdpMetricSink, DEFAULT_PORT};
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let sink = BufferedUdpMetricSink::builder()
    ///     .with_resolver_period(Duration::from_secs(5))
    ///     .with_resolver_error_handler(|e| {
    ///         eprintln!("failed to re-resolve address: {}", e);
    ///     })
    ///     .build(host, socket)
    ///     .unwrap();
    /// ```
    pub fn builder() -> BufferedUdpMetricSinkBuilder {
        BufferedUdpMetricSinkBuilder::new()
    }
}

impl MetricSink for BufferedUdpMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        let mut writer = self.buffer.lock().unwrap();
        writer.write(metric.as_bytes())
    }

    fn flush(&self) -> io::Result<()> {
        let mut writer = self.buffer.lock().unwrap();
        writer.flush()
    }

    fn stats(&self) -> SinkStats {
        (&self.stats).into()
    }
}

#[cfg(test)]
mod tests {
    use super::{BufferedUdpMetricSink, MetricSink, UdpMetricSink};
    use std::net::UdpSocket;

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

        assert_eq!(8, sink.emit("foo:54|c").unwrap());
        assert_eq!(8, sink.emit("foo:67|c").unwrap());
    }

    #[test]
    fn test_buffered_udp_metric_sink_flush() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        // Set the capacity of the buffer such that it won't be flushed
        // from a single write. Thus we can test the flush method.
        let sink = BufferedUdpMetricSink::with_capacity("127.0.0.1:8125", socket, 64).unwrap();

        assert_eq!(8, sink.emit("foo:54|c").unwrap());
        assert!(sink.flush().is_ok());
    }

    #[test]
    fn test_buffered_udp_metric_sink_stats() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let sink = BufferedUdpMetricSink::with_capacity("127.0.0.1:8125", socket, 16).unwrap();

        sink.emit("foo:54|c").unwrap();
        sink.emit("foo:67|c").unwrap();
        sink.flush().unwrap();

        let stats = sink.stats();
        assert!(
            stats.bytes_sent > 0,
            "Expected bytes_sent > 0, got {}",
            stats.bytes_sent
        );
        assert!(
            stats.packets_sent > 0,
            "Expected packets_sent > 0, got {}",
            stats.packets_sent
        );
    }
}

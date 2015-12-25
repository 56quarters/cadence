use log::LogLevel;
use std::io;
use std::net::{
    ToSocketAddrs,
    SocketAddr,
    UdpSocket
};

use types::{
    MetricResult,
    ErrorKind
};

/// Trait for various backends that send Statsd metrics somewhere.
pub trait MetricSink {
    /// Send the Statsd metric using this sink and return the number of bytes
    /// written or an I/O error.
    fn emit(&self, metric: &str) -> io::Result<usize>;
}


/// Implementation of a `MetricSink` that emits metrics over UDP.
///
/// This is the `MetricSink` that almost all consumers of this library will
/// want to use. It accepts a UDP socket instance over which to write metrics
/// and the address of the Statsd server to send packets to.
pub struct UdpMetricSink {
    sink_addr: SocketAddr,
    socket: UdpSocket
}


impl UdpMetricSink {
    /// Construct a new `UdpMetricSink` instance.
    ///
    /// The address should be the address of the remote metric server to
    /// emit metrics to over UDP. The socket should already be bound to a
    /// local address.
    ///
    /// # Example
    ///
    /// ```
    /// use std::net::UdpSocket;
    /// use cadence::{UdpMetricSink, DEFAULT_PORT};
    ///
    /// let host = ("metrics.example.com", DEFAULT_PORT);
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// let sink = UdpMetricSink::new(host, socket);
    /// ```
    ///
    /// # Failures
    ///
    /// This method may fail if:
    ///
    /// * It is unable to resolve the hostname of the metric server.
    /// * The host address is otherwise unable to be parsed
    pub fn new<A>(sink_addr: A, socket: UdpSocket) -> MetricResult<UdpMetricSink>
        where A: ToSocketAddrs {
        // Allow callers to pass anything that implements ToSocketAddrs for
        // convenience but convert it to a concrete address here so that we
        // don't have to pass around the generic parameter everywhere that
        // this sink goes.
        let mut addr_iter = try!(sink_addr.to_socket_addrs());
        let addr = try!(addr_iter.next().ok_or(
            // Tuple that MetricError knows how to be created From
            (ErrorKind::InvalidInput, "No socket addresses yielded")
        ));

        Ok(UdpMetricSink{sink_addr: addr, socket: socket})
    }
}


impl MetricSink for UdpMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.socket.send_to(metric.as_bytes(), &self.sink_addr)
    }
}


/// Implementation of a `MetricSink` that discards all metrics.
///
/// Useful for disabling metric collection or unit tests.
pub struct NopMetricSink;


impl MetricSink for NopMetricSink {
    #[allow(unused_variables)]
    fn emit(&self, metric: &str) -> io::Result<usize> {
        Ok(0)
    }
}


/// Implementation of a `MetricSink` that emits metrics to the console.
///
/// Metrics are emitted with the `println!` macro.
pub struct ConsoleMetricSink;


impl MetricSink for ConsoleMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        println!("{}", metric);
        Ok(metric.len())
    }
}


/// Implementation of a `MetricSink` that emits metrics using the`log!` macro.
///
/// Metrics are emitted using the `LogLevel` provided at construction with a target
/// of `metrics`. Note that the number of bytes written returned by `emit` does not
/// reflect if the provided log level is high enough to be active.
pub struct LoggingMetricSink {
    level: LogLevel
}


impl LoggingMetricSink {
    pub fn new(level: LogLevel) -> LoggingMetricSink {
        LoggingMetricSink{level: level}
    }
}


impl MetricSink for LoggingMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        log!(target: "metrics", self.level, "{}", metric);
        Ok(metric.len())
    }
}


#[cfg(test)]
mod tests {

    use log::LogLevel;
    use std::net::UdpSocket;

    use super::{
        MetricSink,
        NopMetricSink,
        ConsoleMetricSink,
        LoggingMetricSink,
        UdpMetricSink
    };

    // Basic smoke / sanity checks to make sure we're getting
    // some expected results from the various sinks. The UDP
    // sink test isn't really a unit test but *should* just work
    // since it's UDP and it's OK if the packets disapear.

    #[test]
    fn test_nop_metric_sink() {
        let sink = NopMetricSink;
        assert_eq!(0, sink.emit("baz:4|c").unwrap());
    }

    #[test]
    fn test_console_metric_sink() {
        let sink = ConsoleMetricSink;
        assert_eq!(7, sink.emit("foo:2|t").unwrap());
    }

    #[test]
    fn test_logging_metric_sink() {
        let sink = LoggingMetricSink::new(LogLevel::Info);
        assert_eq!(7, sink.emit("bar:1|g").unwrap());
    }

    #[test]
    fn test_udp_metric_sink() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let sink = UdpMetricSink::new("127.0.0.1:8125", socket).unwrap();
        assert_eq!(7, sink.emit("buz:1|m").unwrap());
    }
}

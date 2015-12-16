//!
//!
//!

use log::LogLevel;
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};


///
pub trait MetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize>;
}


///
pub struct UdpMetricSink<A: ToSocketAddrs> {
    sink_addr: A,
    socket: UdpSocket
}


impl<A: ToSocketAddrs> UdpMetricSink<A> {
    pub fn new(sink_addr: A, socket: UdpSocket) -> UdpMetricSink<A> {
        UdpMetricSink{sink_addr: sink_addr, socket: socket}
    }
}


impl<A: ToSocketAddrs> MetricSink for UdpMetricSink<A> {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.socket.send_to(metric.as_bytes(), &self.sink_addr)
    }
}


pub struct NopMetricSink;


impl MetricSink for NopMetricSink {
    #[allow(unused_variables)]
    fn emit(&self, metric: &str) -> io::Result<usize> {
        Ok(0)
    }
}


pub struct ConsoleMetricSink;


impl MetricSink for ConsoleMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        println!("{}", metric);
        Ok(metric.len())
    }
}


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

    use super::{
        MetricSink,
        NopMetricSink,
        ConsoleMetricSink,
        LoggingMetricSink
    };

    // Some basic sanity checks for the debug / test metric
    // sink implementations.

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
    
}

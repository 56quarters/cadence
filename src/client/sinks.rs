//!
//!
//!

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


#[cfg(test)]
mod tests {

    
}

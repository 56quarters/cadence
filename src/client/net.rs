//!
//!
//!

use std::boxed::Box;
use std::io::Error;
use std::net::{ToSocketAddrs, UdpSocket};


// TODO: Should this accept a Metric? Do we need to accept multiple
// metrics and add '\n' for TCP sockets?

///
pub trait MetricSink {
    fn send(&self, buf: &[u8]) -> Result<usize, Error>;
}


///
pub struct UdpMetricSink<A: ToSocketAddrs> {
    sink_addr: Box<A>,
    socket: Box<UdpSocket>
}


impl<A: ToSocketAddrs> UdpMetricSink<A> {
    pub fn new(sink_addr: A, socket: UdpSocket) -> UdpMetricSink<A> {
        UdpMetricSink{sink_addr: Box::new(sink_addr), socket: Box::new(socket)}
    }
}


impl<A: ToSocketAddrs> MetricSink for UdpMetricSink<A> {
    fn send(&self, buf: &[u8]) -> Result<usize, Error> {
        let addr: &A = &self.sink_addr;
        let socket: &UdpSocket = &self.socket;
        socket.send_to(buf, addr)
    }
}


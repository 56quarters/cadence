//!
//!
//!

use std::io::Error;
use std::net::{ToSocketAddrs, SocketAddr, UdpSocket};


///
pub trait MetricSink {
    fn send(&self, buf: &[u8]) -> Result<usize, Error>;
}


///
pub struct UdpMetricSink<'a, A: ToSocketAddrs + 'a> {
    sink_addr: &'a A,
    socket: &'a UdpSocket
}


impl<'a, A: ToSocketAddrs> UdpMetricSink<'a, A> {
    pub fn new(sink_addr: &'a A, socket: &'a UdpSocket) -> UdpMetricSink<'a, A> {
        UdpMetricSink{sink_addr: sink_addr, socket: socket}
    }
}


impl<'a, A: ToSocketAddrs> MetricSink for UdpMetricSink<'a, A> {
    fn send(&self, buf: &[u8]) -> Result<usize, Error> {
        self.socket.send_to(buf, self.sink_addr)
    }
}


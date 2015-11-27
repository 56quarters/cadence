//!
//!
//!

use std::boxed::Box;
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};


///
pub trait MetricSink {
    fn send(&self, metric: &str) -> io::Result<usize>;
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
    fn send(&self, metric: &str) -> io::Result<usize> {
        let addr: &A = &self.sink_addr;
        let socket: &UdpSocket = &self.socket;
        socket.send_to(metric.as_bytes(), addr)
    }
}


pub struct ConsoleMetricSink;


impl MetricSink for ConsoleMetricSink {
    fn send(&self, metric: &str) -> io::Result<usize> {
        println!("{}", metric);
        Ok(0)
    }
}


pub struct NopMetricSink;


impl MetricSink for NopMetricSink {
    #[allow(unused_variables)]
    fn send(&self, metric: &str) -> io::Result<usize> {
        Ok(0)
    }
}



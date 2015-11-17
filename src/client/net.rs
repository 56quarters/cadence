//!
//!
//!

use std::io::Error;
use std::net::{ToSocketAddrs, UdpSocket};

///
pub trait ByteSink {
    fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize, Error>;
}


///
impl ByteSink for UdpSocket {
    fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize, Error> {
        self.send_to(buf, addr)
    }
}

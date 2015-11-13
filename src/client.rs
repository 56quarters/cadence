//
//
//
//
//

use std::ops::Deref;
use std::io::{Error, Write};
use std::net::{ToSocketAddrs, UdpSocket};


pub const DEFAULT_PORT: u16 = 8125;


struct Counter<'a> {
    key: &'a str,
    count: u32,
    sampling: Option<f32>
}


struct Timer<'a> {
    key: &'a str,
    time: u32,
    unit: &'a str,
    sampling: Option<f32>
}


struct Gauge<'a> {
    key: &'a str,
    value: i32
}


trait ToBytes {
    fn to_bytes(&self) -> Box<[u8]>;
}


impl<'a> ToBytes for Counter<'a> {
    fn to_bytes(&self) -> Box<[u8]> {
        let mut v: Vec<u8> = Vec::new();

        match self.sampling {
            Some(val) => write!(&mut v, "{}:{}|c|@{}", self.key, self.count, val),
            None => write!(&mut v, "{}:{}|c", self.key, self.count)
        };

        debug!("Packed vector of {} bytes", v.len());
        v.into_boxed_slice()
    }
}


impl<'a> ToBytes for Timer<'a> {
    fn to_bytes(&self) -> Box<[u8]> {
        let mut v: Vec<u8> = Vec::new();

        match self.sampling {
            Some(val) => write!(&mut v, "{}:{}|{}|@{}", self.key, self.time, self.unit, val),
            None => write!(&mut v, "{}:{}|{}", self.key, self.time, self.unit)
        };

        debug!("Packed vector of {} bytes", v.len());
        v.into_boxed_slice()
    }
}


impl<'a> ToBytes for Gauge<'a> {
    fn to_bytes(&self) -> Box<[u8]> {
        let mut v: Vec<u8> = Vec::new();

        write!(&mut v, "{}:{}|g", self.key, self.value);
        debug!("Packed vector of {} bytes", v.len());
        v.into_boxed_slice()
    }
}


pub trait Counted {
    fn count(&self, key: &str, count: u32, sampling: Option<f32>) -> ();
}


pub trait Timed {
    fn time(&self, key: &str, time: u32, unit: &str, sampling: Option<f32>) -> ();
}


pub trait Gauged {
    fn gauge(&self, key: &str, value: i32) -> ();
}


pub trait ByteSink {
    fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize, Error>;
}


impl ByteSink for UdpSocket {
    fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize, Error> {
        self.send_to(buf, addr)
    }
}


pub struct StatsdClient<'a, T: ByteSink + 'a> {
    host: &'a str,
    port: u16,
    prefix: &'a str,
    sink: &'a T
}


impl<'a, T: ByteSink> StatsdClient<'a, T> {
    pub fn from_host(
        host: &'a str,
        port: u16,
        prefix: &'a str,
        sink: &'a T) -> StatsdClient<'a, T> {

        StatsdClient{
            host: host,
            port: port,
            prefix: prefix,
            sink: sink
        }
    }

    fn send_metric<B: ToBytes>(&self, metric: B) -> () {
        let bytes = metric.to_bytes();
        let addr = (self.host, self.port);
        debug!("Sending to {}:{}", self.host, self.port);

        match self.sink.send_to(bytes.deref(), addr) {
            Ok(n) => debug!("Wrote {} bytes to socket", n),
            Err(err) => debug!("Got error writing to socket: {}", err)
        }
    }
}


fn make_key(prefix: &str, key: &str) -> String {
    let trimmed_prefix = if prefix.ends_with('.') {
        prefix.trim_right_matches('.')
    } else {
        prefix
    };

    format!("{}.{}", trimmed_prefix, key)
}


impl<'a, T: ByteSink> Counted for StatsdClient<'a, T> {
    fn count(&self, key: &str, count: u32, sampling: Option<f32>) -> () {
        let counter = Counter{
            key: &make_key(self.prefix, key), count: count, sampling: sampling};
        self.send_metric(counter);
    }
}


impl<'a, T: ByteSink> Timed for StatsdClient<'a, T> {
    fn time(&self, key: &str, time: u32, unit: &str, sampling: Option<f32>) -> () {
        let timer = Timer{
            key: &make_key(self.prefix, key), time: time, unit: unit, sampling: sampling};
        self.send_metric(timer);
    }
}


impl<'a, T: ByteSink> Gauged for StatsdClient<'a, T> {
    fn gauge(&self, key: &str, value: i32) -> () {
        let gauge = Gauge{key: &make_key(self.prefix, key), value: value};
        self.send_metric(gauge);
    }
}


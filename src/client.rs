//
//
//
//
//

use std::net::UdpSocket;


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
    fn to_bytes(&self) -> &[u8];
}


impl<'a> ToBytes for Counter<'a> {
    fn to_bytes(&self) -> &[u8] {
        &[]
    }
}


impl<'a> ToBytes for Timer<'a> {
    fn to_bytes(&self) -> &[u8] {
        &[]
    }
}


impl<'a> ToBytes for Gauge<'a> {
    fn to_bytes(&self) -> &[u8] {
        &[]
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


pub struct StatsdClientUdp<'a> {
    host: &'a str,
    port: u16,
    prefix: &'a str
}


impl<'a> StatsdClientUdp<'a> {
    pub fn from_host(host: &'a str, port: u16, prefix: &'a str) -> StatsdClientUdp<'a> {
        StatsdClientUdp{
            host: host,
            port: port,
            prefix: prefix
        }
    }
}


impl<'a> Counted for StatsdClientUdp<'a> {
    fn count(&self, key: &str, count: u32, sampling: Option<f32>) -> () {
        println!("counted!")
    }
}


impl<'a> Timed for StatsdClientUdp<'a> {
    fn time(&self, key: &str, time: u32, unit: &str, sampling: Option<f32>) -> () {
        println!("timed!")
    }
}


impl<'a> Gauged for StatsdClientUdp<'a> {
    fn gauge(&self, key: &str, value: i32) -> () {
        println!("gauged!")
    }
}


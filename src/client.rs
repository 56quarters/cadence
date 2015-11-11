//
//
//
//
//

use std::net::UdpSocket;


pub const DEFAULT_PORT: u16 = 8125;


pub struct Counter<'a> {
    pub key: &'a str,
    pub count: u32,
    pub sampling: Option<f32>
}


pub struct Timer<'a> {
    pub key: &'a str,
    pub time: u32,
    pub unit: &'a str,
    pub sampling: Option<f32>
}


pub struct Gauge<'a> {
    pub key: &'a str,
    pub value: i32
}


pub trait Counted {
    fn count(&self, c: &Counter) -> ();
}


pub trait Timed {
    fn time(&self, t: &Timer) -> ();
}


pub trait Gauged {
    fn gauge(&self, g: &Gauge) -> ();
}


pub struct StatsdClientUdp<'a> {
    socket: &'a mut UdpSocket,
    prefix: &'a str
}


impl<'a> StatsdClientUdp<'a> {
    pub fn from_socket(socket: &'a mut UdpSocket, prefix: &'a str) -> StatsdClientUdp<'a> {
        StatsdClientUdp{
            prefix: prefix, socket: socket
        }
    }
}


impl<'a> Counted for StatsdClientUdp<'a> {
    fn count(&self, c: &Counter) -> () {
        println!("counted!")
    }
}


impl<'a> Timed for StatsdClientUdp<'a> {
    fn time(&self, t: &Timer) -> () {
        println!("timed!")
    }
}


impl<'a> Gauged for StatsdClientUdp<'a> {
    fn gauge(&self, g: &Gauge) -> () {
        println!("gauged!")
    }
}


extern crate statsd;

use std::net::UdpSocket;
use std::string::String;

use statsd::*;


fn main() {
    println!("This is the thing!!");
    let mut socket = UdpSocket::bind(("localhost", DEFAULT_PORT)).unwrap();
    let client = StatsdClientUdp::from_socket(&mut socket, "foo.prefix");
    
    client.count(&Counter{key: "some.key", count: 4, sampling: None});
    client.gauge(&Gauge{key: "some.key", value: 42});
    client.time(&Timer{key: "some.key", time: 15, unit: "ms", sampling: None});
}


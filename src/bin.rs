extern crate statsd;

use std::net::UdpSocket;
use std::string::String;

use statsd::*;


fn main() {
    println!("This is the thing!!");
    let client = StatsdClientUdp::from_host("localhost", DEFAULT_PORT, "foo.prefix");
    
    client.count("some.key", 4, None);
    client.gauge("some.key", 42);
    client.time("some.key", 15, "ms", None);
}


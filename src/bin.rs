extern crate statsd;

use std::net::UdpSocket;

use statsd::client::*;


fn main() {
    println!("This is the thing!!");
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let client = StatsdClient::from_host("127.0.0.1", DEFAULT_PORT, "foo.prefix", &socket);
    client.count("some.key", 4, None);
    client.gauge("some.key", 42);
    client.time("some.key", 15, "ms", None);
}


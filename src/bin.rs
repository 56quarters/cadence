extern crate statsd;

use std::net::UdpSocket;

use statsd::client::*;


fn main() {
    println!("This is the thing!!");

    let metric_host = ("127.0.0.1", DEFAULT_PORT);
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = UdpMetricSink::new(&metric_host, &socket);
    let client = StatsdClient::new("foo.prefix", &sink);
    client.count("some.key", 4, None);
    client.gauge("some.key", 42);
    client.time("some.key", 15, None);
}


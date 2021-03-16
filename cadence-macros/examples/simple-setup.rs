// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
use cadence_macros::{statsd_count, statsd_distribution, statsd_gauge, statsd_histogram, statsd_meter, statsd_set, statsd_time};
use std::net::UdpSocket;

fn main() {
    let sock = UdpSocket::bind("0.0.0.0:0").unwrap();
    let sink = UdpMetricSink::from(("localhost", DEFAULT_PORT), sock).unwrap();
    let client = StatsdClient::from_sink("example.prefix", sink);
    cadence_macros::set_global_default(client);

    statsd_count!("some.counter", 1, "tag" => "val");
    statsd_gauge!("some.gauge", 1, "tag" => "val");
    statsd_time!("some.timer", 1, "tag" => "val");
    statsd_meter!("some.meter", 1, "tag" => "val");
    statsd_histogram!("some.histogram", 1, "tag" => "val");
    statsd_distribution!("some.distribution", 1, "tag" => "val");
    statsd_set!("some.set", 1, "tag" => "val");
}

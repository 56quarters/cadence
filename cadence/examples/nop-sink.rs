// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows how the Cadence client could be used with a 'no-op' sink
// that just discards all metrics. This might be useful if you want to disable
// metric collection for some reason.

use cadence::prelude::*;
use cadence::{NopMetricSink, StatsdClient};

fn main() {
    let sink = NopMetricSink;
    let metrics = StatsdClient::from_sink("example.prefix", sink);

    metrics.count("example.counter", 1).unwrap();
    metrics.gauge("example.gauge", 5).unwrap();
    metrics.time("example.timer", 32).unwrap();
    metrics.histogram("example.histogram", 22).unwrap();
    metrics.distribution("example.distribution", 33).unwrap();
    metrics.meter("example.meter", 8).unwrap();
}

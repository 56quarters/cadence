// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows how you set a timestamp, sampling rate or container id
// as described on https://docs.datadoghq.com/developers/dogstatsd/datagram_shell/.

use cadence::prelude::*;
use cadence::{MetricError, NopMetricSink, StatsdClient};

fn main() {
    fn my_error_handler(err: MetricError) {
        eprintln!("Error sending metrics: {}", err);
    }

    // Create a client with an error handler and default "region" tag
    let client = StatsdClient::builder("my.prefix", NopMetricSink)
        .with_error_handler(my_error_handler)
        .with_container_id("container-123")
        .build();

    // In this case we are sending a counter metric with manually set timestamp,
    // container id and sampling rate. If sending the metric fails, our error
    // handler set above will be invoked to do something with the metric error.
    client
        .count_with_tags("counter.1", 1)
        .with_timestamp(123456)
        .with_container_id("container-456")
        .with_sampling_rate(0.5)
        .send();

    // In this case we are sending the same counter metrics without any explicit container
    // id, meaning that the client's container id will be used.
    let res = client
        .count_with_tags("counter.2", 1)
        .try_send();

    println!("Result of metric send: {:?}", res);
}

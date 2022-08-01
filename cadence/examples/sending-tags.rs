// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows how you might use Cadence to send DataDog-style tags
// either with a method the returns the result of sending them, or with a
// method the delegates any errors to a predefined error handler. It also
// includes "default" tags which are automatically added to any metrics
// sent.

use cadence::prelude::*;
use cadence::{MetricError, NopMetricSink, StatsdClient};

fn main() {
    fn my_error_handler(err: MetricError) {
        eprintln!("Error sending metrics: {}", err);
    }

    // Create a client with an error handler and default "region" tag
    let client = StatsdClient::builder("my.prefix", NopMetricSink)
        .with_error_handler(my_error_handler)
        .with_tag("region", "us-west-2")
        .build();

    // In this case we are sending a counter metric with two tag key-value
    // pairs. If sending the metric fails, our error handler set above will
    // be invoked to do something with the metric error.
    client
        .count_with_tags("requests.handled", 1)
        .with_tag("app", "search")
        .with_tag("user", "1234")
        .send();

    // In this case we are sending the same counter metrics with two tags.
    // The results of sending the metric (or failing to send it) are returned
    // to the caller to do something with.
    let res = client
        .count_with_tags("requests.handled", 1)
        .with_tag("app", "search")
        .with_tag("user", "1234")
        .try_send();

    println!("Result of metric send: {:?}", res);
}

// Cadence - An extensible Statsd client for Rust!
//
// This file is dual-licensed to the public domain and under the following
// license: you are granted a perpetual, irrevocable license to copy, modify,
// publish, and distribute this file as you see fit.

// This example shows how you might use Cadence to send Datadog-style tags
// either with a method the returns the result of sending them, or with a
// method the delegates any errors to a predefined error handler.

use cadence::prelude::*;
use cadence::{MetricError, NopMetricSink, StatsdClient};

fn main() {
    fn my_error_handler(err: MetricError) -> () {
        println!("Error sending metrics: {}", err);
    }

    let client = StatsdClient::builder("my.prefix", NopMetricSink)
        .with_error_handler(my_error_handler)
        .build();

    // In this case we are sending a counter metric with two tag key-value
    // pairs. If sending the metric fails, our error handler set above will
    // be invoked to do something with the metric error.
    client
        .incr_with_tags("requests.handled")
        .with_tag("app", "search")
        .with_tag("region", "us-west-2")
        .send();

    // In this case we are sending the same counter metrics with two tags.
    // The results of sending the metric (or failing to send it) are returned
    // to the caller to do something with.
    let res = client
        .incr_with_tags("requests.handled")
        .with_tag("app", "search")
        .with_tag("region", "us-west-2")
        .try_send();

    println!("Result of metric send: {:?}", res);
}

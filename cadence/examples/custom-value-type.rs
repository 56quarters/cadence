// Cadence - An extensible Statsd client for Rust!
//
// To the extent possible under law, the author(s) have dedicated all copyright and
// related and neighboring rights to this file to the public domain worldwide.
// This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along with this
// software. If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.

// This example shows how to use the various To*Value traits in Cadence to allow
// custom types to be used as metric values.

use cadence::ext::{MetricValue, ToGaugeValue};
use cadence::prelude::*;
use cadence::{MetricResult, NopMetricSink, StatsdClient};

enum UserHappiness {
    VeryHappy,
    KindaHappy,
    Sad,
}

impl ToGaugeValue for UserHappiness {
    fn try_to_value(self) -> MetricResult<MetricValue> {
        let v = match self {
            UserHappiness::VeryHappy => 1.0,
            UserHappiness::KindaHappy => 0.5,
            UserHappiness::Sad => 0.0,
        };

        Ok(MetricValue::Float(v))
    }
}

fn main() {
    let sink = NopMetricSink;
    let client = StatsdClient::from_sink("example.prefix", sink);

    client.gauge("user.happiness", UserHappiness::VeryHappy).unwrap();
    client.gauge("user.happiness", UserHappiness::KindaHappy).unwrap();
    client.gauge("user.happiness", UserHappiness::Sad).unwrap();
}

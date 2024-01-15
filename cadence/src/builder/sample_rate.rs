use super::{byte_str::ByteStr, MetricType};
use crate::{ErrorKind, MetricError};
use std::fmt::Write;

/// Represents the sample rate of a metric. This is used to determine how often
/// a metric should be sent to the the statsd server. The sample rate is a value
/// between 0.0 and 1.0.
///
/// > A float between 0 and 1, inclusive. Only works with COUNT, HISTOGRAM,
/// > DISTRIBUTION, and TIMER metrics. The default is 1, which samples 100% of the
/// > time.
/// > - via [DataDog](https://docs.datadoghq.com/developers/dogstatsd/datagram_shell)
#[derive(Debug, Clone, Copy)]
pub(crate) struct SampleRate {
    value: f32,
    outbuf: ByteStr<8>, // 8 bytes is enough for "@{value}", so we don't need to allocate
}

impl SampleRate {
    const MIN_SIZE: usize = 3;

    fn new(value: f32) -> Self {
        let mut outbuf = ByteStr::<8>::new();
        write!(&mut outbuf, "@{:.6}", value).expect("failed to write sample rate");
        Self::trim(&mut outbuf);

        Self { value, outbuf }
    }

    pub fn is_applicable_to_metric(&self, metric_type: MetricType) -> bool {
        match metric_type {
            MetricType::Counter | MetricType::Histogram | MetricType::Distribution | MetricType::Timer => {
                self.value != 1.0
            }
            _ => false,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn as_str(&self) -> &str {
        self.outbuf.as_str()
    }

    #[allow(dead_code)]
    pub fn kv_size(&self) -> usize {
        self.outbuf.len()
    }

    fn trim<const N: usize>(bytestr: &mut ByteStr<N>) {
        loop {
            if bytestr.len() <= Self::MIN_SIZE {
                break;
            }

            if !bytestr.chomp_trailing_byte(b'0') {
                break;
            }
        }
    }
}

impl TryFrom<f32> for SampleRate {
    type Error = MetricError;

    fn try_from(rate: f32) -> Result<Self, Self::Error> {
        if rate > 0.0 && rate <= 1.0 {
            Ok(Self::new(rate))
        } else {
            let err = MetricError::from((ErrorKind::InvalidInput, "Sample rate must be between 0.0 and 1.0"));
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        builder::{sample_rate::SampleRate, MetricFormatter},
        ext::MetricValue,
        Counter, MetricBuilder, NopMetricSink, SpyMetricSink, StatsdClient,
    };

    #[test]
    fn test_metric_formatter_counter_with_sample_rate() {
        let mut fmt = MetricFormatter::counter("prefix.", "some.key", MetricValue::Signed(4));
        fmt.with_sample_rate(0.5);

        assert_eq!("prefix.some.key:4|c|@0.5", &fmt.format())
    }

    #[test]
    fn test_metric_formatter_counter_with_sample_rate_rounding() {
        let mut fmt = MetricFormatter::counter("prefix.", "some.key", MetricValue::Signed(4));
        fmt.with_sample_rate(1.0 / 54.0);

        assert_eq!("prefix.some.key:4|c|@0.01851", &fmt.format())
    }

    #[test]
    fn test_sample_rate_kv_size() {
        for _ in 0..1000 {
            let random_float = rand::random::<f32>();
            let sr = SampleRate::try_from(random_float).unwrap();
            let result = sr.as_str();
            assert_eq!(sr.kv_size(), result.len(), "sample rate was: {}, dbg: {:?}", result, sr);
        }
    }

    #[test]
    fn test_doesnt_write_default_sample_rate() {
        let mut fmt = MetricFormatter::counter("prefix.", "some.key", MetricValue::Signed(4));
        fmt.with_sample_rate(1.0);

        assert_eq!("prefix.some.key:4|c", &fmt.format())
    }

    #[test]
    fn test_only_writes_when_applicable_to_metric() {
        let mut counter = MetricFormatter::counter("prefix.", "some.key", MetricValue::Signed(4));
        counter.with_sample_rate(0.5);
        let mut histogram = MetricFormatter::histogram("prefix.", "some.key", MetricValue::Float(3.15));
        histogram.with_sample_rate(0.5);
        let mut timer = MetricFormatter::timer("prefix.", "some.key", MetricValue::Float(3.15));
        timer.with_sample_rate(0.5);
        let mut distribution = MetricFormatter::distribution("prefix.", "some.key", MetricValue::Float(3.15));
        distribution.with_sample_rate(0.5);

        assert_eq!("prefix.some.key:4|c|@0.5", &counter.format());
        assert_eq!("prefix.some.key:3.15|h|@0.5", &histogram.format());
        assert_eq!("prefix.some.key:3.15|ms|@0.5", &timer.format());
        assert_eq!("prefix.some.key:3.15|d|@0.5", &distribution.format());

        let mut set = MetricFormatter::set("prefix.", "some.key", MetricValue::Signed(4));
        set.with_sample_rate(0.5);
        let mut gauge = MetricFormatter::gauge("prefix.", "some.key", MetricValue::Signed(4));
        gauge.with_sample_rate(0.5);

        assert_eq!("prefix.some.key:4|s", &set.format());
        assert_eq!("prefix.some.key:4|g", &gauge.format());
    }

    #[test]
    fn test_metric_builder_try_send_actually_samples() {
        let rx = {
            let (rx, sink) = SpyMetricSink::new();
            let client = StatsdClient::builder("prefix.", sink).build();
            let shared_client = Arc::new(client);

            let _ = (0..10)
                .map(|_| {
                    let local_client = shared_client.clone();

                    std::thread::spawn(move || {
                        for i in 0..10 {
                            let mut fmt = MetricFormatter::counter("prefix.", "some.counter", MetricValue::Signed(i));
                            fmt.with_sample_rate(0.5);
                            let builder: MetricBuilder<'_, '_, Counter> = MetricBuilder::from_fmt(fmt, &local_client);

                            builder.try_send().unwrap();
                            std::thread::yield_now();
                        }
                    })
                })
                .map(|t| t.join().unwrap())
                .collect::<Vec<_>>();

            rx
        };

        let sent_metrics: Vec<_> = rx.iter().collect();

        assert!(!sent_metrics.is_empty()); // always happening (probably)
        assert!(sent_metrics.len() < 100); // never happening (probably)
    }

    #[test]
    fn test_try_send_with_sample_rate_success() {
        let mut fmt = MetricFormatter::counter("prefix.", "some.counter", MetricValue::Signed(11));
        fmt.with_sample_rate(0.5);
        let client = StatsdClient::from_sink("prefix.", NopMetricSink);
        let builder: MetricBuilder<'_, '_, Counter> = MetricBuilder::from_fmt(fmt, &client);
        let res = builder.try_send();

        assert!(res.is_ok(), "expected Ok result from try_send");
    }
}

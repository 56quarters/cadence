// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2020 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::io::MultiLineWriter;
use crate::sinks::core::MetricSink;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::fmt::{self, Debug, Formatter};
use std::panic::RefUnwindSafe;

// Default size of the buffer for buffered metric sinks, picked for
// consistency with the UDP implementation.
const DEFAULT_BUFFER_SIZE: usize = 512;

#[derive(Clone)]
struct SpyWriter {
    inner: Arc<Mutex<dyn Write + Send + RefUnwindSafe + 'static>>,
}

impl SpyWriter {
    fn from(inner: Arc<Mutex<dyn Write + Send + RefUnwindSafe + 'static>>) -> Self {
        SpyWriter { inner }
    }
}

impl Write for SpyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut writer = self.inner.lock().unwrap();
        writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut writer = self.inner.lock().unwrap();
        writer.flush()
    }
}

/// `MetricSink` implementation that writes all metrics to a shared `Write`
/// instance that callers retain a reference to.
///
/// This is not a general purpose sink, rather it's a sink meant for verifying
/// metrics written during the course of integration tests. Due to the requirement
/// that callers retain a shared reference to the underlying `Write` implementation,
/// this sink uses more locking (mutexes) than other sinks in Cadence. Thus, it
/// should not be used in production, only testing.
///
/// Each metric is sent to the underlying writer when the `.emit()` method is
/// called, in the thread of the caller.
pub struct SpyMetricSink {
    writer: Mutex<SpyWriter>,
}

impl SpyMetricSink {
    pub fn from(writer: Arc<Mutex<dyn Write + Send + RefUnwindSafe + 'static>>) -> Self {
        SpyMetricSink {
            writer: Mutex::new(SpyWriter::from(writer))
        }
    }
}

impl MetricSink for SpyMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        let mut writer = self.writer.lock().unwrap();
        writer.write(metric.as_bytes())
    }
}

impl Debug for SpyMetricSink {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SpyMetricSink {{ Mutex {{ SpyWriter {{ ... }} }} }}")
    }
}

/// `MetricSink` implementation that buffers metrics and writes them to a
/// shared `Write` instance that callers retain a reference to.
///
/// This is not a general purpose sink, rather it's a sink meant for verifying
/// metrics written during the course of integration tests. Due to the requirement
/// that callers retain a shared reference to the underlying `Write` implementation,
/// this sink uses more locking (mutexes) than other sinks in Cadence. Thus, it
/// should not be used in production, only testing.
///
/// Metrics are line buffered, meaning that a trailing "\n" is added
/// after each metric written to this sink. When the buffer is sufficiently
/// full and a write is attempted, the contents of the buffer are flushed to
/// the underlying writer and then the metric is written to the buffer. The
/// buffer is also flushed when this sink is destroyed.
///
/// The default size of the buffer is 512 bytes. This is to be consistent with
/// the default for the `BufferedUdpMetricSink`. The buffer size can be customized
/// using the `with_capacity` method to create the sink if desired.
///
/// If a metric larger than the buffer is emitted, it will be written
/// directly to the underlying writer, bypassing the buffer.
///
/// Note that since metrics are buffered until a certain size is reached, it's
/// possible that they may sit in the buffer for a while for applications
/// that do not emit metrics frequently or at a high volume. For these low-
/// throughput use cases, it may make more sense to use the `SpyMetricSink`
/// since it sends metrics immediately with no buffering.
pub struct BufferedSpyMetricSink {
    writer: Mutex<MultiLineWriter<SpyWriter>>,
}

impl BufferedSpyMetricSink {
    pub fn from(writer: Arc<Mutex<dyn Write + Send + RefUnwindSafe + 'static>>) -> Self {
        Self::with_capacity(writer, DEFAULT_BUFFER_SIZE)
    }

    pub fn with_capacity(writer: Arc<Mutex<dyn Write + Send + RefUnwindSafe + 'static>>, cap: usize) -> Self {
        BufferedSpyMetricSink {
            writer: Mutex::new(MultiLineWriter::new(
                SpyWriter::from(writer),
                cap,
            ))
        }
    }
}

impl MetricSink for BufferedSpyMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        let mut writer = self.writer.lock().unwrap();
        writer.write(metric.as_bytes())
    }

    fn flush(&self) -> io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()
    }
}

impl Debug for BufferedSpyMetricSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BufferedSpyMetricSink {{ Mutex {{ MultiLineWriter {{ SpyWriter {{ ... }} }} }} }}")
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use super::{BufferedSpyMetricSink, MetricSink, SpyMetricSink};

    // Get a copy of the contents of the shared writer and make sure to
    // drop the lock before any assertions, otherwise the mutex becomes
    // poisoned and results in obscure errors when trying to debug tests
    #[inline]
    fn copy_buffer(writer: Arc<Mutex<Vec<u8>>>) -> Vec<u8> {
        writer.lock().unwrap().clone()
    }

    #[test]
    fn test_spy_metric_sink() {
        let writer = Arc::new(Mutex::new(Vec::new()));
        let sink = SpyMetricSink::from(writer.clone());
        sink.emit("buz:1|c").unwrap();

        let contents = copy_buffer(writer);
        assert_eq!("buz:1|c".as_bytes(), contents.as_slice());
    }

    #[test]
    fn test_buffered_spy_metric_sink() {
        let writer = Arc::new(Mutex::new(Vec::new()));

        // Make sure the sink is dropped before checking what was written
        // to the buffer so that we know everything was flushed
        {
            let sink = BufferedSpyMetricSink::with_capacity(writer.clone(), 16);
            sink.emit("foo:54|c").unwrap();
            sink.emit("foo:67|c").unwrap();
        }

        let contents = copy_buffer(writer);
        assert_eq!("foo:54|c\nfoo:67|c\n".as_bytes(), contents.as_slice());
    }

    #[test]
    fn test_buffered_spy_metric_sink_flush() {
        let writer = Arc::new(Mutex::new(Vec::new()));

        // Set the capacity of the buffer such that it won't be flushed
        // from a single write. Thus we can test the flush method.
        let sink = BufferedSpyMetricSink::with_capacity(writer.clone(), 64);
        sink.emit("foo:54|c").unwrap();
        sink.emit("foo:67|c").unwrap();
        let flush = sink.flush();

        let contents = copy_buffer(writer);
        assert_eq!("foo:54|c\nfoo:67|c\n".as_bytes(), contents.as_slice());
        assert!(flush.is_ok());
    }
}

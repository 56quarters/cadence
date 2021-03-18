// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2020-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::io::MultiLineWriter;
use crate::sinks::core::MetricSink;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender, TrySendError};
use std::io::{self, ErrorKind, Write};
use std::sync::Mutex;

// Default size of the buffer for buffered metric sinks, picked for
// consistency with the UDP implementation.
const DEFAULT_BUFFER_SIZE: usize = 512;

/// `MetricSink` implementation that writes all metrics to the `Sender` half of
/// a channel while callers are given ownership of the `Receiver` half.
///
/// This is not a general purpose sink, rather it's a sink meant for verifying
/// metrics written during the course of integration tests. By default, the channel
/// used is unbounded. The channel size can be limited using the `with_capacity` method.
///
/// Each metric is sent to the underlying channel when the `.emit()` method is
/// called, in the thread of the caller.
#[derive(Debug)]
pub struct SpyMetricSink {
    sender: Sender<Vec<u8>>,
}

impl SpyMetricSink {
    pub fn new() -> (Receiver<Vec<u8>>, Self) {
        Self::with_queue_capacity(None)
    }

    pub fn with_capacity(queue: usize) -> (Receiver<Vec<u8>>, Self) {
        Self::with_queue_capacity(Some(queue))
    }

    fn with_queue_capacity(queue: Option<usize>) -> (Receiver<Vec<u8>>, Self) {
        let (tx, rx) = new_channel(queue);
        let sink = SpyMetricSink { sender: tx };
        (rx, sink)
    }
}

impl MetricSink for SpyMetricSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        send_metric(&self.sender, metric.as_bytes())
    }
}

/// `MetricSink` implementation that buffers metrics and writes them to the
/// `Sender` half of a channel while callers are given ownership of the `Receiver`
/// half.
///
/// This is not a general purpose sink, rather it's a sink meant for verifying
/// metrics written during the course of integration tests. By default, the channel
/// used is unbounded. The channel size can be limited using the `with_capacity` method.
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
#[derive(Debug)]
pub struct BufferedSpyMetricSink {
    writer: Mutex<MultiLineWriter<WriteAdapter>>,
}

impl BufferedSpyMetricSink {
    pub fn new() -> (Receiver<Vec<u8>>, Self) {
        Self::with_capacity(None, Some(DEFAULT_BUFFER_SIZE))
    }

    pub fn with_capacity(queue: Option<usize>, buffer: Option<usize>) -> (Receiver<Vec<u8>>, Self) {
        let (tx, rx) = new_channel(queue);
        let buffer_sz = buffer.unwrap_or(DEFAULT_BUFFER_SIZE);
        let writer = MultiLineWriter::new(WriteAdapter::new(tx), buffer_sz);
        let sink = BufferedSpyMetricSink {
            writer: Mutex::new(writer),
        };
        (rx, sink)
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

#[derive(Debug)]
struct WriteAdapter {
    sender: Sender<Vec<u8>>,
}

impl WriteAdapter {
    fn new(sender: Sender<Vec<u8>>) -> Self {
        WriteAdapter { sender }
    }
}

impl Write for WriteAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        send_metric(&self.sender, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn new_channel(cap: Option<usize>) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {
    if let Some(sz) = cap {
        bounded(sz)
    } else {
        unbounded()
    }
}

fn send_metric(sender: &Sender<Vec<u8>>, metric: &[u8]) -> io::Result<usize> {
    match sender.try_send(metric.to_vec()) {
        Err(TrySendError::Disconnected(_)) => Err(io::Error::new(ErrorKind::Other, "channel disconnected")),
        Err(TrySendError::Full(_)) => Err(io::Error::new(ErrorKind::Other, "channel full")),
        Ok(_) => Ok(metric.len()),
    }
}

#[cfg(test)]
mod test {
    use super::{BufferedSpyMetricSink, MetricSink, SpyMetricSink};

    #[test]
    fn test_spy_metric_sink() {
        let (rx, sink) = SpyMetricSink::new();
        sink.emit("buz:1|c").unwrap();

        let sent = rx.recv().unwrap();
        assert_eq!("buz:1|c".as_bytes(), sent.as_slice());
    }

    #[test]
    fn test_buffered_spy_metric_sink() {
        // Make sure the sink is dropped before checking what was written
        // to the buffer so that we know everything was flushed
        let rx = {
            let (rx, sink) = BufferedSpyMetricSink::with_capacity(None, Some(64));
            sink.emit("foo:54|c").unwrap();
            sink.emit("foo:67|c").unwrap();
            rx
        };

        let sent = rx.recv().unwrap();
        assert_eq!("foo:54|c\nfoo:67|c\n".as_bytes(), sent.as_slice());
    }

    #[test]
    fn test_buffered_spy_metric_sink_flush() {
        // Set the capacity of the buffer such that it won't be flushed
        // from a single write. Thus we can test the flush method.
        let (rx, sink) = BufferedSpyMetricSink::with_capacity(None, Some(64));
        sink.emit("foo:54|c").unwrap();
        sink.emit("foo:67|c").unwrap();
        let flush = sink.flush();

        let sent = rx.recv().unwrap();
        assert_eq!("foo:54|c\nfoo:67|c\n".as_bytes(), sent.as_slice());
        assert!(flush.is_ok());
    }
}

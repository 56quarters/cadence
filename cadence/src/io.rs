// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2021 Nick Pillitteri
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{self, WriterPanicked};
use std::io::{BufWriter, Write};
use std::str;

#[derive(Debug, Default)]
struct WriterMetrics {
    inner_write: u64,
    buf_write: u64,
    flushed: u64,
}

/// Buffered implementation of the `Write` trait that appends a
/// trailing line ending string to every input written and only
/// writes the complete input in a single call to the underlying
/// writer.
#[derive(Debug)]
pub struct MultiLineWriter<T>
where
    T: Write,
{
    written: usize,
    capacity: usize,
    metrics: WriterMetrics,
    inner: BufWriter<T>,
    line_ending: Vec<u8>,
}

impl<T> MultiLineWriter<T>
where
    T: Write,
{
    /// Create a new buffered `MultiLineWriter` instance that suffixes
    /// each write with a newline character ('\n').
    pub fn new(inner: T, cap: usize) -> MultiLineWriter<T> {
        Self::with_ending(inner, cap, "\n")
    }

    /// Create a new buffered `MultiLineWriter` instance that suffixes
    /// each write with the given line ending.
    pub fn with_ending(inner: T, cap: usize, end: &str) -> MultiLineWriter<T> {
        MultiLineWriter {
            written: 0,
            capacity: cap,
            metrics: WriterMetrics::default(),
            inner: BufWriter::with_capacity(cap, inner),
            line_ending: Vec::from(end.as_bytes()),
        }
    }

    /// Gets a reference to the underlying writer.
    pub fn get_ref(&self) -> &T {
        self.inner.get_ref()
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    /// Replace the underlying writer.
    ///
    /// Returns the parts of the buffered original writer.
    /// See [`std::io::BufWriter::into_parts`].
    pub fn replace_writer(&mut self, writer: T) -> (T, Result<Vec<u8>, WriterPanicked>) {
        // Create a new buffered writer with the same capacity
        let buf_writer = BufWriter::with_capacity(self.capacity, writer);
        // Replace the inner writer
        let orig = std::mem::replace(&mut self.inner, buf_writer);
        // Reset state
        self.written = 0;

        orig.into_parts()
    }

    #[allow(dead_code)]
    fn get_metrics(&self) -> &WriterMetrics {
        &self.metrics
    }
}

impl<T> Write for MultiLineWriter<T>
where
    T: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let left = self.capacity - self.written;
        let required = buf.len() + self.line_ending.len();

        if required > self.capacity {
            // if buffer non-empty, flush to preserve ordering.
            if self.written > 0 {
                self.flush()?;
            }
            self.metrics.inner_write += 1;
            // If the user has given us a value bigger than our buffer
            // to write, bypass the buffer and write directly to the Write
            // implementation that our BufWriter is wrapping. Note that we
            // don't write a trailing newline in this case. The reasoning
            // is that the newlines are separators for putting multiple
            // "things" into a single write call to the underlying impl
            // (probably a UDP socket). Thus, there's no value in adding
            // a newline when we're only writing a single large value to
            // the underlying impl.
            // See https://github.com/56quarters/cadence/issues/87
            Ok(self.inner.get_mut().write(buf)?)
        } else {
            if left < required {
                self.flush()?;
            }

            self.metrics.buf_write += 1;
            // Perform the buffered write of user data and the trailing
            // newlines. Increment the number of bytes written to the
            // buffer after each write in case they return errors.
            let write1 = self.inner.write(buf)?;
            self.written += write1;

            let write2 = self.inner.write(&self.line_ending)?;
            self.written += write2;

            // We keep track of the total number of bytes written above but
            // we only return the number of bytes from the provided buffer we
            // wrote per the `Write::write` contract.
            // See https://github.com/56quarters/cadence/issues/117
            Ok(write1)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.metrics.flushed += 1;
        self.inner.flush()?;
        self.written = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MultiLineWriter;

    use std::io::{self, Write};
    use std::{cmp, panic, str};

    /// A mock writer that panics after writing a specified number of bytes.
    /// Used for testing panic recovery in buffered writers.
    #[derive(Debug)]
    struct PanickingWriter {
        buffer: Vec<u8>,
        panic_after_bytes: usize,
    }

    impl PanickingWriter {
        fn new(panic_after_bytes: usize) -> Self {
            Self {
                buffer: Vec::new(),
                panic_after_bytes,
            }
        }

        fn buffer(&self) -> &[u8] {
            &self.buffer
        }
    }

    impl Write for PanickingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let bytes_until_panic = self.panic_after_bytes.saturating_sub(self.buffer.len());
            let bytes_to_write = cmp::min(buf.len(), bytes_until_panic);

            self.buffer.extend_from_slice(&buf[..bytes_to_write]);

            // panic if we did not write the full buffer
            if buf.len() > bytes_to_write {
                panic!("PanickingWriter triggered a panic");
            }

            Ok(bytes_to_write)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn with_newline(data: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(data);
        buf.push(b'\n');
        buf
    }

    #[test]
    fn test_write_needs_flush() {
        let mut buffered = MultiLineWriter::new(vec![], 16);

        let write1 = buffered.write(b"foo:1234|c").unwrap();
        let written_after_write1 = buffered.get_ref().len();

        let write2 = buffered.write(b"baz:5678|c").unwrap();
        let written_after_write2 = buffered.get_ref().len();

        let written = str::from_utf8(buffered.get_ref()).unwrap();

        assert_eq!(10, write1);
        assert_eq!(0, written_after_write1);

        assert_eq!(10, write2);
        assert_eq!(11, written_after_write2);

        assert_eq!("foo:1234|c\n", written);
    }

    #[test]
    fn test_write_no_flush() {
        let mut buffered = MultiLineWriter::new(vec![], 32);

        let write1 = buffered.write(b"abc:3|g").unwrap();
        let written_after_write1 = buffered.get_ref().len();

        let write2 = buffered.write(b"def:4|g").unwrap();
        let written_after_write2 = buffered.get_ref().len();

        assert_eq!(7, write1);
        assert_eq!(0, written_after_write1);

        assert_eq!(7, write2);
        assert_eq!(0, written_after_write2);
    }

    #[test]
    fn test_write_bigger_than_buffer() {
        let mut buffered = MultiLineWriter::new(vec![], 16);

        let write1 = buffered.write(b"some_really_long_metric:456|c").unwrap();
        let written_after_write1 = buffered.get_ref().len();
        let in_buffer_after_write1 = buffered.written;

        let write2 = buffered.write(b"abc:4|g").unwrap();
        let written_after_write2 = buffered.get_ref().len();
        let in_buffer_after_write2 = buffered.written;

        assert_eq!(29, write1);
        assert_eq!(29, written_after_write1);
        assert_eq!(0, in_buffer_after_write1);

        assert_eq!(7, write2);
        assert_eq!(29, written_after_write2);
        assert_eq!(8, in_buffer_after_write2);
    }

    #[test]
    fn test_buffer_write_equal_capacity() {
        let mut buffered = MultiLineWriter::new(vec![], 8);

        let bytes_written = buffered.write(b"foo:42|c").unwrap();
        let written = str::from_utf8(buffered.get_ref()).unwrap();
        let buf_metrics = buffered.get_metrics();

        assert_eq!("foo:42|c", written);
        assert_eq!(8, bytes_written, "expected {} bytes", 8);
        assert_eq!(1, buf_metrics.inner_write, "expected inner_write = {}", 1);
        assert_eq!(0, buf_metrics.buf_write, "expected buf_write = {}", 0);
        assert_eq!(0, buf_metrics.flushed, "expected flushed = {}", 0);
    }

    #[test]
    fn test_flush_still_buffered() {
        let mut buffered = MultiLineWriter::new(vec![], 32);

        buffered.write_all(b"xyz").unwrap();
        buffered.write_all(b"abc").unwrap();
        let len_after_writes = buffered.get_ref().len();

        buffered.flush().unwrap();
        let written = str::from_utf8(buffered.get_ref()).unwrap();

        assert_eq!(0, len_after_writes);
        assert_eq!("xyz\nabc\n", written);
    }

    #[test]
    fn test_buffer_flushed_when_dropped() {
        let mut buf: Vec<u8> = vec![];

        // Create our writer in a different scope to ensure that the
        // BufWriter it's using internally is flushed when it goes out
        // of scope and anything that was buffered gets written out.
        {
            let mut writer = MultiLineWriter::new(&mut buf, 32);
            writer.write_all(b"something").unwrap();
            assert_eq!(0, writer.get_ref().len());
        }

        assert_eq!(10, buf.len());
        assert_eq!("something\n", str::from_utf8(&buf).unwrap());
    }

    #[test]
    fn test_multiline_writer_ordering_behavior() {
        // Create a buffer with capacity = 20 bytes
        let buffer_capacity = 20;
        let writer = Vec::new();
        let mut buffered = MultiLineWriter::new(writer, buffer_capacity);

        // Small string that fits in buffer
        let small_metric = "small:1234|c";
        assert!(small_metric.len() + buffered.line_ending.len() <= buffer_capacity);

        // Large string that exceeds buffer capacity
        let large_metric = "this_is_a_very_long_metric_name:9876|c";
        assert!(large_metric.len() > buffer_capacity);

        // Write the small metric (should be buffered with newline)
        buffered.write_all(small_metric.as_bytes()).unwrap();

        // Nothing should be written yet (still in buffer)
        assert_eq!(0, buffered.get_ref().len());

        // Write the large metric (should bypass buffer with no newline)
        buffered.write_all(large_metric.as_bytes()).unwrap();

        // The large metric should be written immediately
        // The small metric should be flushed before writing the large one
        let result = str::from_utf8(buffered.get_ref()).unwrap();

        // Check that small metric has a newline but large metric doesn't
        assert_eq!(format!("{}\n{}", small_metric, large_metric), result);

        // Confirm metrics handling in buffered writer
        let metrics = buffered.get_metrics();
        assert_eq!(1, metrics.inner_write); // direct write count (large metric)
        assert_eq!(1, metrics.buf_write); // buffered write count (small metric)
        assert_eq!(1, metrics.flushed); // flush count (before large write)
    }

    #[test]
    fn test_replace_writer() {
        let cap = 32;
        let first_data = b"first data";
        assert!(first_data.len() < cap);

        // Setup: Create a MultiLineWriter with a Vec<u8> as the inner writer and write first data chunk
        let mut buffered = MultiLineWriter::new(Vec::new(), cap);
        buffered.write_all(first_data).unwrap();
        // force a flush to ensure inner writer is reached
        buffered.flush().unwrap();

        // write second chunk of data
        let second_data = b"second data";
        assert!(second_data.len() < cap);
        buffered.write_all(second_data).unwrap();

        // Create a new Vec<u8> to replace the existing one
        let (original_writer, buffer_result) = buffered.replace_writer(Vec::new());
        let original_buffer = buffer_result.expect("BufWriter not in panicked state");

        // Verify buffer states
        assert_eq!(with_newline(first_data), original_writer);
        assert_eq!(with_newline(second_data), original_buffer);

        // Verify internal state was reset
        assert_eq!(buffered.written, 0);

        // Write to the new buffer and verify it works
        let third_data = b"third data";
        buffered.write_all(third_data).unwrap();
        buffered.flush().unwrap();

        // Verify that the new writer received the data
        assert_eq!(with_newline(third_data).as_slice(), buffered.get_ref());
    }

    #[test]
    fn test_replace_writer_with_writer_panicked() {
        // Define the panic position - after 5 bytes written
        let panic_position = 5;
        let cap = 32;

        // first chunk at boundary of panic_position
        let first_write = b"12345";
        assert!(first_write.len() == panic_position);

        // second chunk where first + second > panic_position but < cap
        // This will trigger a panic during a flush not a write
        let second_write = b"abcd";
        let total_write_len = first_write.len() + second_write.len();
        assert!(total_write_len > panic_position);
        assert!(total_write_len < cap);

        // Create a writer that will panic after writing the specified number of bytes
        let writer = PanickingWriter::new(panic_position);

        // Create our MultiLineWriter with the panicking writer
        let mut buffered = MultiLineWriter::new(writer, cap);

        // writing both chunks are buffered and succeed
        buffered.write_all(first_write).unwrap();
        buffered.write_all(second_write).unwrap();

        // The flush should fail with a panic
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| buffered.flush()));
        assert!(result.is_err());

        // Now replace the writer
        let new_writer = PanickingWriter::new(usize::MAX);
        let (original_writer, buffer_result) = buffered.replace_writer(new_writer);

        // We should recover a WriterPanicked error
        let err = buffer_result.expect_err("WriterPanicked");
        assert_eq!(
            [with_newline(first_write), with_newline(second_write)].concat(),
            err.into_inner()
        );

        // The original writer should contain the bytes written before the panic
        assert_eq!(original_writer.buffer(), first_write);

        // Verify internal state was reset
        assert_eq!(buffered.written, 0);

        // Verify we can still use the new writer
        let new_data = b"new data";
        buffered.write_all(new_data).unwrap();
        buffered.flush().unwrap();

        // Verify the new writer received the data
        assert_eq!(with_newline(new_data).as_slice(), buffered.get_ref().buffer());
    }
}

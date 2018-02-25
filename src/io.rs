// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2017 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::io::{BufWriter, Write};
use std::net::{SocketAddr, UdpSocket};
use std::str;

#[derive(Debug)]
struct WriterMetrics {
    inner_write: u64,
    buf_write: u64,
    flushed: u64,
}

impl WriterMetrics {
    fn new() -> Self {
        WriterMetrics {
            inner_write: 0,
            buf_write: 0,
            flushed: 0,
        }
    }
}

/// Buffered implementation of the `Write` trait that appends a
/// trailing line ending string to every input written and only
/// writes the complete input in a single call to the underlying
/// writer.
#[derive(Debug)]
pub(crate) struct MultiLineWriter<T: Write> {
    written: usize,
    capacity: usize,
    metrics: WriterMetrics,
    inner: BufWriter<T>,
    line_ending: Vec<u8>,
}

impl<T: Write> MultiLineWriter<T> {
    /// Create a new buffered `MultiLineWriter` instance that suffixes
    /// each write with a newline character ('\n').
    pub(crate) fn new(cap: usize, inner: T) -> MultiLineWriter<T> {
        Self::with_ending(cap, inner, "\n")
    }

    /// Create a new buffered `MultiLineWriter` instance that suffixes
    /// each write with the given line ending.
    pub(crate) fn with_ending(cap: usize, inner: T, end: &str) -> MultiLineWriter<T> {
        MultiLineWriter {
            written: 0,
            capacity: cap,
            metrics: WriterMetrics::new(),
            inner: BufWriter::with_capacity(cap, inner),
            line_ending: Vec::from(end.as_bytes()),
        }
    }

    #[allow(dead_code)]
    fn get_ref(&self) -> &T {
        self.inner.get_ref()
    }

    #[allow(dead_code)]
    fn get_metrics(&self) -> &WriterMetrics {
        &self.metrics
    }
}

impl<T: Write> Write for MultiLineWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let left = self.capacity - self.written;
        let required = buf.len() + self.line_ending.len();

        if required > self.capacity {
            self.metrics.inner_write += 1;
            // If the user has given us a value bigger than our buffer
            // to write, bypass the buffer and write directly to the Write
            // implementation that our BufWriter is wrapping.
            let write1 = self.inner.get_mut().write(buf)?;
            let write2 = self.inner.get_mut().write(&self.line_ending)?;
            Ok(write1 + write2)
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
            Ok(write1 + write2)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.metrics.flushed += 1;
        self.inner.flush()?;
        self.written = 0;
        Ok(())
    }
}

/// Adapter for writing to a `UdpSocket` via the `Write` trait
#[derive(Debug)]
pub(crate) struct UdpWriteAdapter {
    addr: SocketAddr,
    socket: UdpSocket,
}

impl UdpWriteAdapter {
    pub(crate) fn new(addr: SocketAddr, socket: UdpSocket) -> UdpWriteAdapter {
        UdpWriteAdapter {
            addr: addr,
            socket: socket,
        }
    }
}

impl Write for UdpWriteAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send_to(buf, &self.addr)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MultiLineWriter;

    use std::str;
    use std::io::Write;

    #[test]
    fn test_write_needs_flush() {
        let mut buffered = MultiLineWriter::new(16, vec![]);

        let write1 = buffered.write("foo:1234|c".as_bytes()).unwrap();
        let written_after_write1 = buffered.get_ref().len();

        let write2 = buffered.write("baz:56789|c".as_bytes()).unwrap();
        let written_after_write2 = buffered.get_ref().len();

        let written = str::from_utf8(&buffered.get_ref()).unwrap();

        assert_eq!(11, write1);
        assert_eq!(0, written_after_write1);

        assert_eq!(12, write2);
        assert_eq!(11, written_after_write2);

        assert_eq!("foo:1234|c\n", written);
    }

    #[test]
    fn test_write_no_flush() {
        let mut buffered = MultiLineWriter::new(32, vec![]);

        let write1 = buffered.write("abc:3|g".as_bytes()).unwrap();
        let written_after_write1 = buffered.get_ref().len();

        let write2 = buffered.write("def:4|g".as_bytes()).unwrap();
        let written_after_write2 = buffered.get_ref().len();

        assert_eq!(8, write1);
        assert_eq!(0, written_after_write1);

        assert_eq!(8, write2);
        assert_eq!(0, written_after_write2);
    }

    #[test]
    fn test_write_bigger_than_buffer() {
        let mut buffered = MultiLineWriter::new(16, vec![]);

        let write1 = buffered
            .write("some_really_long_metric:456|c".as_bytes())
            .unwrap();
        let written_after_write1 = buffered.get_ref().len();
        let in_buffer_after_write1 = buffered.written;

        let write2 = buffered.write("abc:4|g".as_bytes()).unwrap();
        let written_after_write2 = buffered.get_ref().len();
        let in_buffer_after_write2 = buffered.written;

        assert_eq!(30, write1);
        assert_eq!(30, written_after_write1);
        assert_eq!(0, in_buffer_after_write1);

        assert_eq!(8, write2);
        assert_eq!(30, written_after_write2);
        assert_eq!(8, in_buffer_after_write2);
    }

    // Make sure that writing an 8 byte payload to a buffer with only
    // 8 bytes of capacity results in using the "direct write" code
    // path since we need to take the trailing newline into account
    #[test]
    fn test_buffer_write_equal_capacity() {
        let mut buffered = MultiLineWriter::new(8, vec![]);

        let bytes_written = buffered.write("foo:42|c".as_bytes()).unwrap();
        let written = str::from_utf8(&buffered.get_ref()).unwrap();
        let buf_metrics = buffered.get_metrics();

        assert_eq!("foo:42|c\n", written);
        assert_eq!(9, bytes_written, "expected {} bytes", 9);
        assert_eq!(1, buf_metrics.inner_write, "expected inner_write = {}", 1);
        assert_eq!(0, buf_metrics.buf_write, "expected buf_write = {}", 0);
        assert_eq!(0, buf_metrics.flushed, "expected flushed = {}", 0);
    }

    #[test]
    fn test_flush_still_buffered() {
        let mut buffered = MultiLineWriter::new(32, vec![]);

        buffered.write("xyz".as_bytes()).unwrap();
        buffered.write("abc".as_bytes()).unwrap();
        let len_after_writes = buffered.get_ref().len();

        buffered.flush().unwrap();
        let written = str::from_utf8(&buffered.get_ref()).unwrap();

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
            let mut writer = MultiLineWriter::new(32, &mut buf);
            let _r = writer.write("something".as_bytes()).unwrap();
            assert_eq!(0, writer.get_ref().len());
        }

        assert_eq!(10, buf.len());
        assert_eq!("something\n", str::from_utf8(&buf).unwrap());
    }
}

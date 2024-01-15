use std::fmt;

/// A fixed size byte string buffer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ByteStr<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> ByteStr<N> {
    pub fn new() -> Self {
        ByteStr { bytes: [0; N], len: 0 }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes())
            .expect("failed to convert byte string to utf8 string, this should never happen")
    }

    /// Take the give str, and fills the buffer up, truncating if necessary.
    pub fn extend_from_slice<T: AsRef<[u8]>>(&mut self, slice: T) {
        let bytes = slice.as_ref();
        let slice_to_fill = self.bytes[self.len..].iter_mut();

        for (i, byte) in slice_to_fill.enumerate() {
            if let Some(b) = bytes.get(i) {
                *byte = *b;
                self.len = self.len.saturating_add(1);
            } else {
                break;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len as _
    }

    pub fn chomp_trailing_byte(&mut self, byte: u8) -> bool {
        if self.as_bytes().last() == Some(&byte) {
            self.len -= 1;
            true
        } else {
            false
        }
    }
}

// Capture the output of `write!` into fixed size buffer.
impl<const N: usize> fmt::Write for ByteStr<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.extend_from_slice(s);
        Ok(())
    }
}

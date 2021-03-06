use std::os::raw::c_char;

use crate::error::{check_status, Error, ErrorKind};

/// Initial number of bytes to allocate when copying a string out of a string vector
const INITIAL_SIZE: usize = 128;
/// Maximum number of bytes to allocate when copying a string out of a string vector
const MAX_SIZE: usize = 1024 * 1024;

/// A helper for copying a string from a C API
///
/// operation should be a function that takes a pointer to a buffer of characters and the length
/// of the buffer. It calls a C function that fills the buffer and returns an error code.
///
/// If the error code indicates an error, the error is returned. Otherwise, this function attempts
/// to convert the copied characters into a String.
///
/// This function returns errors in the following cases:
/// * `operation` returned an error code that indicates an error
/// * The string to be copied is longer than the maximum allowed length
/// * The string copied is not valid UTF-8
pub(crate) fn copy_string<F>(mut operation: F) -> Result<String, Error>
where
    F: FnMut(*mut c_char, usize) -> uhd_sys::uhd_error::Type,
{
    let mut buffer: Vec<u8> = Vec::new();
    for size in BufferSizes::new() {
        buffer.resize(size, b'\0');

        // Call into the C code to copy the string
        let status = operation(buffer.as_mut_ptr() as *mut c_char, buffer.len());
        check_status(status)?;

        // Get the part of the buffer before the first null
        if let Some(null_index) = buffer.iter().position(|b| *b == b'\0') {
            buffer.truncate(null_index);
            buffer.shrink_to_fit();
            // Try to convert to UTF-8
            return String::from_utf8(buffer).map_err(|_| Error::new(ErrorKind::Utf8));
        } else {
            // If there is no null, the error message was longer than BUFFER_LENGTH.
            // Try again with the next size.
            continue;
        }
    }
    // String is too large to fully copy
    Err(Error::new(ErrorKind::StringLength))
}

/// An iterator over buffer sizes that yields INITIAL_SIZE and then double the previous value
/// up to MAX_SIZE
struct BufferSizes {
    /// The next value to return
    next: usize,
}

impl BufferSizes {
    pub fn new() -> Self {
        BufferSizes { next: INITIAL_SIZE }
    }
}

impl Iterator for BufferSizes {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next > MAX_SIZE {
            // Maximum exceeded
            None
        } else {
            let current = self.next;
            self.next *= 2;
            Some(current)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_sizes() {
        let mut sizes = BufferSizes::new();
        assert_eq!(Some(128), sizes.next());
        assert_eq!(Some(256), sizes.next());
        assert_eq!(Some(512), sizes.next());
        assert_eq!(Some(1024), sizes.next());
        assert_eq!(Some(2048), sizes.next());
        assert_eq!(Some(4096), sizes.next());
        assert_eq!(Some(8192), sizes.next());
        assert_eq!(Some(16384), sizes.next());
        assert_eq!(Some(32768), sizes.next());
        assert_eq!(Some(65536), sizes.next());
        assert_eq!(Some(131072), sizes.next());
        assert_eq!(Some(262144), sizes.next());
        assert_eq!(Some(524288), sizes.next());
        assert_eq!(Some(1048576), sizes.next());
        assert_eq!(None, sizes.next());
    }
}

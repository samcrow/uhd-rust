use std::convert::TryInto;
use std::ffi::{CString, NulError};
use std::ptr;

use crate::check_status;
use crate::error::{Error, ErrorKind};
use crate::utils::copy_string;

/// A handle to a std::vector of std::strings
pub(crate) struct StringVector(uhd_sys::uhd_string_vector_handle);

impl StringVector {
    pub fn new() -> Result<Self, Error> {
        let mut handle: uhd_sys::uhd_string_vector_handle = ptr::null_mut();
        let status = unsafe { uhd_sys::uhd_string_vector_make(&mut handle) };
        match Error::from_code_with_last_error(status) {
            Some(e) => Err(e),
            None => Ok(StringVector(handle)),
        }
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        let status = unsafe { uhd_sys::uhd_string_vector_size(self.0, &mut len) };
        check_status(status).unwrap();
        len.try_into().expect("Length does not fit into usize")
    }

    /// Appends a string to the end of this vector
    ///
    /// This function returns an error if the provided value contains a null byte.
    ///
    /// max_len: The maximum
    pub fn append(&mut self, value: &str) -> Result<(), NulError> {
        let value_c = CString::new(value)?;
        let status = unsafe { uhd_sys::uhd_string_vector_push_back(&mut self.0, value_c.as_ptr()) };
        check_status(status).unwrap();
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<Result<String, Error>> {
        let status = copy_string(|buffer, length| unsafe {
            uhd_sys::uhd_string_vector_at(self.0, index as _, buffer, length as _)
        });
        match status {
            Ok(value) => Some(Ok(value)),
            Err(e) => match e.kind() {
                ErrorKind::StdExcept => {
                    // This is most likely an std::out_of_range because the index was >= length.
                    None
                }
                _ => Some(Err(e)),
            },
        }
    }

    /// Returns an iterator over the items in this vector
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            vector: self,
            next: 0,
            length: self.len(),
        }
    }

    /// Returns the underlying handle
    pub(crate) fn handle_mut(&mut self) -> &mut uhd_sys::uhd_string_vector_handle {
        &mut self.0
    }
}

impl Drop for StringVector {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_string_vector_free(&mut self.0) };
    }
}

/// An iterator over items in a string vector
pub struct Iter<'v> {
    /// Vector being iterated over
    vector: &'v StringVector,
    /// Index of next item to yield (invariant: next <= length)
    next: usize,
    /// Number of items in vector
    length: usize,
}

impl Iterator for Iter<'_> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.length {
            None
        } else {
            let item = self.vector.get(self.next)?;
            self.next += 1;
            Some(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.length - self.next;
        (size, Some(size))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.length - self.next
    }
}

impl ExactSizeIterator for Iter<'_> {}

impl<'v> IntoIterator for &'v StringVector {
    type Item = <Iter<'v> as Iterator>::Item;
    type IntoIter = Iter<'v>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl From<StringVector> for Vec<String> {
    fn from(strings: StringVector) -> Self {
        From::from(&strings)
    }
}
impl From<&'_ StringVector> for Vec<String> {
    fn from(strings: &StringVector) -> Self {
        strings
            .iter()
            .flat_map(|string_or_err| string_or_err.ok())
            .collect()
    }
}

mod fmt {
    use super::StringVector;
    use std::fmt::*;

    impl Debug for StringVector {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_list()
                .entries(self.iter().map(|item| {
                    // Item may be a normal String or an invalid UTF-8 error
                    item.unwrap_or_else(|_| "<invalid UTF-8>".to_owned())
                }))
                .finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_vector_empty() -> Result<(), Box<dyn std::error::Error>> {
        let vector = StringVector::new()?;
        assert_eq!(None, vector.get(0));
        Ok(())
    }

    #[test]
    fn string_vector_small() -> Result<(), Box<dyn std::error::Error>> {
        let mut vector = StringVector::new()?;
        assert_eq!(None, vector.get(0));
        let value0 = "slithy toves";
        vector.append(value0)?;
        assert_eq!(Some(Ok(value0.to_owned())), vector.get(0));
        Ok(())
    }

    #[test]
    fn string_vector_large() -> Result<(), Box<dyn std::error::Error>> {
        let mut vector = StringVector::new()?;
        assert_eq!(None, vector.get(0));
        let value0 = "mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths \
        outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe mome raths outgrabe \
        mome raths outgrabe mome raths outgrabe mome raths outgrabe ";
        vector.append(value0)?;
        assert_eq!(Some(Ok(value0.to_owned())), vector.get(0));
        Ok(())
    }
}

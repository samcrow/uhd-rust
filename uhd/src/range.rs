use crate::error::{check_status, Error};
use std::ptr;

/// A range of floating-point values, and a step-by amount
#[derive(Clone)]
pub struct Range(uhd_sys::uhd_range_t);

impl Default for Range {
    fn default() -> Self {
        Range(uhd_sys::uhd_range_t {
            start: 0.0,
            stop: 0.0,
            step: 0.0,
        })
    }
}

/// A list of ranges of floating-point values
///
/// The ranges in a meta-range should be monotonic (the start of each range should be greater
/// than or equal to the end of the preceding range). Gaps between ranges are allowed.
///
/// Most MetaRange methods will return errors if called on a non-monotonic range.
pub struct MetaRange(uhd_sys::uhd_meta_range_handle);

impl MetaRange {
    /// Creates an empty meta-range
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the overall start of this meta-range
    pub fn start(&self) -> Result<f64, Error> {
        let mut start = 0.0;
        check_status(unsafe { uhd_sys::uhd_meta_range_start(self.0, &mut start) })?;
        Ok(start)
    }

    /// Returns the overall end (stop) of this meta-range
    pub fn stop(&self) -> Result<f64, Error> {
        let mut stop = 0.0;
        check_status(unsafe { uhd_sys::uhd_meta_range_stop(self.0, &mut stop) })?;
        Ok(stop)
    }

    /// Returns the "overall step value" of this meta-range (the minimum of the step values of
    /// each contained range, and the gaps between ranges)
    pub fn step(&self) -> Result<f64, Error> {
        let mut step = 0.0;
        check_status(unsafe { uhd_sys::uhd_meta_range_step(self.0, &mut step) })?;
        Ok(step)
    }

    /// Returns the number of ranges in this meta-range
    pub fn len(&self) -> usize {
        let mut length = 0usize;
        check_status(unsafe {
            uhd_sys::uhd_meta_range_size(self.0, &mut length as *mut usize as *mut _)
        })
        .unwrap();
        length
    }

    /// Returns the range at the provided index, if one exists
    pub fn get(&self, index: usize) -> Option<Range> {
        let mut range = Range::default();
        match check_status(unsafe { uhd_sys::uhd_meta_range_at(self.0, index as _, &mut range.0) })
        {
            Ok(()) => Some(range),
            Err(e) => match e {
                // StdExcept usually indicates a std::out_of_range because index >= length
                Error::StdExcept => None,
                _ => panic!("Unexpected UHD error: {}", e),
            },
        }
    }
    /// Appends a range to the end of this meta-range
    pub fn push(&mut self, range: Range) {
        check_status(unsafe { uhd_sys::uhd_meta_range_push_back(self.0, &range.0) }).unwrap();
    }

    /// Returns an iterator over ranges in this meta-range
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            range: self,
            next: 0,
            length: self.len(),
        }
    }

    pub(crate) fn handle(&mut self) -> uhd_sys::uhd_meta_range_handle {
        self.0
    }
}

impl Default for MetaRange {
    /// Creates an empty meta-range
    fn default() -> Self {
        let mut handle = ptr::null_mut();
        check_status(unsafe { uhd_sys::uhd_meta_range_make(&mut handle) }).unwrap();
        MetaRange(handle)
    }
}

impl Drop for MetaRange {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_meta_range_free(&mut self.0) };
    }
}

impl<'m> IntoIterator for &'m MetaRange {
    type Item = Range;
    type IntoIter = Iter<'m>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over ranges in a meta-range
pub struct Iter<'m> {
    range: &'m MetaRange,
    /// The index of the next element to yield
    /// Invariant: next <= length
    next: usize,
    length: usize,
}

impl Iterator for Iter<'_> {
    type Item = Range;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.length {
            None
        } else {
            let item = self.range.get(self.next).unwrap();
            self.next += 1;
            Some(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.length - self.next;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.length - self.next
    }
}

impl ExactSizeIterator for Iter<'_> {}

mod fmt {
    use super::{MetaRange, Range};
    use std::fmt::{Debug, Formatter, Result};

    impl Debug for MetaRange {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_list().entries(self.iter()).finish()
        }
    }

    impl Debug for Range {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_struct("Range")
                .field("start", &self.0.start)
                .field("stop", &self.0.stop)
                .field("step", &self.0.step)
                .finish()
        }
    }
}

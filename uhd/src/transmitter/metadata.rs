use std::ptr;

use crate::error::check_status;

use crate::TimeSpec;

/// Data about a transmit operation
pub struct TransmitMetadata {
    /// Handle to C++ object
    handle: uhd_sys::uhd_tx_metadata_handle,
    /// Number of samples transmitted
    samples: usize,
}

impl TransmitMetadata {
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the timestamp of (the first?) of the transmitted samples, according to the USRP's
    /// internal clock
    pub fn time_spec(&self) -> Option<TimeSpec> {
        if self.has_time_spec() {
            let mut time = TimeSpec::default();
            let mut seconds_time_t: libc::time_t = Default::default();

            check_status(unsafe {
                uhd_sys::uhd_tx_metadata_time_spec(
                    self.handle,
                    &mut seconds_time_t,
                    &mut time.fraction,
                )
            })
            .unwrap();
            // Convert seconds from time_t to i64
            time.seconds = seconds_time_t.into();
            Some(time)
        } else {
            None
        }
    }

    /// Returns true if this metadata object has a time
    fn has_time_spec(&self) -> bool {
        let mut has = false;
        check_status(unsafe { uhd_sys::uhd_tx_metadata_has_time_spec(self.handle, &mut has) })
            .unwrap();
        has
    }

    /// Returns true if the transmitted samples are at the beginning of a burst
    pub fn start_of_burst(&self) -> bool {
        let mut value = false;
        check_status(unsafe { uhd_sys::uhd_tx_metadata_start_of_burst(self.handle, &mut value) })
            .unwrap();
        value
    }

    /// Returns true if the transmitted samples are at the end of a burst
    pub fn end_of_burst(&self) -> bool {
        let mut value = false;
        check_status(unsafe { uhd_sys::uhd_tx_metadata_end_of_burst(self.handle, &mut value) })
            .unwrap();
        value
    }

    /// Returns the number of samples transmitted
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Sets the number of samples transmitted
    pub(crate) fn set_samples(&mut self, samples: usize) {
        self.samples = samples
    }

    pub(crate) fn handle_mut(&mut self) -> &mut uhd_sys::uhd_tx_metadata_handle {
        &mut self.handle
    }
}

// Thread safety: The uhd_tx_metadata struct just stores data. All exposed functions read fields.
unsafe impl Send for TransmitMetadata {}
unsafe impl Sync for TransmitMetadata {}

impl Default for TransmitMetadata {
    fn default() -> Self {
        let mut handle: uhd_sys::uhd_tx_metadata_handle = ptr::null_mut();

        // not sure what to do here, need to look at docs
        let has_time_spec = Default::default();
        let full_secs = Default::default();
        let frac_secs = Default::default();
        let start_of_burst = Default::default();
        let end_of_burst = Default::default();

        check_status(unsafe {
            uhd_sys::uhd_tx_metadata_make(
                &mut handle,
                has_time_spec,
                full_secs,
                frac_secs,
                start_of_burst,
                end_of_burst,
            )
        })
        .unwrap();
        TransmitMetadata { handle, samples: 0 }
    }
}

impl Drop for TransmitMetadata {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_tx_metadata_free(&mut self.handle) };
    }
}

mod fmt {
    use super::TransmitMetadata;
    use std::fmt::{Debug, Formatter, Result};

    impl Debug for TransmitMetadata {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_struct("TransmitMetadata")
                .field("time_spec", &self.time_spec())
                .field("start_of_burst", &self.start_of_burst())
                .field("end_of_burst", &self.end_of_burst())
                .field("received_samples", &self.samples())
                .finish()
        }
    }
}

#[cfg(test)]
mod test {
    use super::TransmitMetadata;

    #[test]
    fn default_tx_metadata() {
        let metadata = TransmitMetadata::default();
        assert_eq!(None, metadata.time_spec());
        assert_eq!(false, metadata.start_of_burst());
        assert_eq!(false, metadata.end_of_burst());
    }
}

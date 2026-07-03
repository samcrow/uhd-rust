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

    /// Build a `TransmitMetadata` with explicit burst flags and an optional time spec.
    ///
    /// Useful for burst transmissions. Per the Ettus documentation, the first packet of a
    /// burst should carry `start_of_burst = true` and the last packet should carry
    /// `end_of_burst = true` so the radio can bracket the burst cleanly (RF enabled
    /// at SOB, disabled at EOB, fresh re-init on the next SOB).
    ///
    /// `Default::default()` constructs metadata with all three fields cleared, which
    /// is appropriate for continuous-streaming workloads.
    pub fn with_flags(
        start_of_burst: bool,
        end_of_burst: bool,
        time_spec: Option<TimeSpec>,
    ) -> Self {
        let (has_time_spec, full_secs, frac_secs) = match time_spec {
            Some(t) => (true, t.seconds, t.fraction),
            None => (false, 0i64, 0.0f64),
        };
        let mut handle: uhd_sys::uhd_tx_metadata_handle = ptr::null_mut();
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

    /// Returns the timestamp of (the first?) of the transmitted samples, according to the USRP's
    /// internal clock
    #[allow(clippy::useless_conversion)]
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

            // Explicitly convert seconds from time_t to i64 (some platforms `time_t` is smaller
            // than `i64`)
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
        assert!(!metadata.start_of_burst());
        assert!(!metadata.end_of_burst());
    }

    #[test]
    fn with_flags_sob_eob_round_trip() {
        let md = TransmitMetadata::with_flags(true, true, None);
        assert!(md.start_of_burst());
        assert!(md.end_of_burst());
        assert_eq!(None, md.time_spec());
    }

    #[test]
    fn with_flags_time_spec_round_trip() {
        use crate::TimeSpec;
        let md = TransmitMetadata::with_flags(
            true,
            false,
            Some(TimeSpec {
                seconds: 42,
                fraction: 0.125,
            }),
        );
        assert!(md.start_of_burst());
        assert!(!md.end_of_burst());
        let t = md.time_spec().expect("time_spec should be Some");
        assert_eq!(42, t.seconds);
        assert!((t.fraction - 0.125).abs() < 1e-12);
    }
}

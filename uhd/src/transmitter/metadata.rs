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

    // /// Returns true if the provided transmit buffer was not large enough to hold a full packet
    // ///
    // /// If this is the case, the fragment_offset() function returns the offset from the beginning
    // /// of the packet to the first sample transmitted
    // pub fn more_fragments(&self) -> bool {
    //     let mut value = false;
    //     check_status(unsafe { uhd_sys::uhd_tx_metadata_more_fragments(self.handle, &mut value) })
    //         .unwrap();
    //     value
    // }

    // /// If more_fragments() returned true, this function returns the offset from the beginning
    // /// of the packet to the first sample transmitted
    // pub fn fragment_offset(&self) -> usize {
    //     let mut value = 0usize;
    //     check_status(unsafe {
    //         uhd_sys::uhd_tx_metadata_fragment_offset(
    //             self.handle,
    //             &mut value as *mut usize as *mut _,
    //         )
    //     })
    //     .unwrap();
    //     value
    // }

    // /// Returns true if a packet was dropped or transmitted out of order
    // pub fn out_of_sequence(&self) -> bool {
    //     let mut value = false;
    //     check_status(unsafe { uhd_sys::uhd_tx_metadata_out_of_sequence(self.handle, &mut value) })
    //         .unwrap();
    //     value
    // }

    /// Returns the number of samples transmitted
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Sets the number of samples transmitted
    pub(crate) fn set_samples(&mut self, samples: usize) {
        self.samples = samples
    }

    // /// Returns the error code associated with the transmit operation
    // fn error_code(&self) -> uhd_sys::uhd_tx_metadata_error_code_t::Type {
    //     let mut code = uhd_sys::uhd_tx_metadata_error_code_t::UHD_tx_METADATA_ERROR_CODE_NONE;
    //     check_status(unsafe { uhd_sys::uhd_tx_metadata_error_code(self.handle, &mut code) })
    //         .unwrap();
    //     code
    // }

    // /// Returns the error associated with the transmit operation, if any
    // pub fn last_error(&self) -> Option<TransmitError> {
    //     let out_of_sequence = self.out_of_sequence();
    //     use uhd_sys::uhd_tx_metadata_error_code_t::*;
    //     let kind = match self.error_code() {
    //         UHD_tx_METADATA_ERROR_CODE_TIMEOUT => TransmitErrorKind::Timeout,
    //         UHD_tx_METADATA_ERROR_CODE_LATE_COMMAND => TransmitErrorKind::LateCommand,
    //         UHD_tx_METADATA_ERROR_CODE_BROKEN_CHAIN => TransmitErrorKind::BrokenChain,
    //         UHD_tx_METADATA_ERROR_CODE_OVERFLOW if !out_of_sequence => TransmitErrorKind::Overflow,
    //         UHD_tx_METADATA_ERROR_CODE_OVERFLOW if out_of_sequence => {
    //             TransmitErrorKind::OutOfSequence
    //         }
    //         UHD_tx_METADATA_ERROR_CODE_ALIGNMENT => TransmitErrorKind::Alignment,
    //         UHD_tx_METADATA_ERROR_CODE_BAD_PACKET => TransmitErrorKind::BadPacket,
    //         UHD_tx_METADATA_ERROR_CODE_NONE => {
    //             // Not actually an error
    //             return None;
    //         }
    //         _ => {
    //             // Some other error
    //             TransmitErrorKind::Other
    //         }
    //     };
    //     let message = copy_string(|buffer, length| unsafe {
    //         uhd_sys::uhd_tx_metadata_strerror(self.handle, buffer, length as _)
    //     })
    //     .ok();

    //     Some(TransmitError { kind, message })
    // }

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
    use super::*;
    use super::{TransmitError, TransmitMetadata};
    use std::fmt::{Debug, Display, Formatter, Result};

    impl Debug for TransmitMetadata {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_struct("TransmitMetadata")
                .field("time_spec", &self.time_spec())
                // .field("more_fragments", &self.more_fragments())
                // .field("fragment_offset", &self.fragment_offset())
                .field("start_of_burst", &self.start_of_burst())
                .field("end_of_burst", &self.end_of_burst())
                .finish()
        }
    }

    impl Display for TransmitError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self.kind {
                TransmitErrorKind::Timeout => write!(f, "No packet transmitted"),
                TransmitErrorKind::LateCommand => write!(f, "Command timestamp was in the past"),
                TransmitErrorKind::BrokenChain => write!(f, "Expected another stream command"),
                TransmitErrorKind::Overflow => {
                    write!(f, "An internal transmit buffer has been filled")
                }
                TransmitErrorKind::OutOfSequence => write!(f, "Sequence error"),
                TransmitErrorKind::Alignment => write!(f, "Multi-channel alignment failed"),
                TransmitErrorKind::BadPacket => write!(f, "A packet could not be parsed"),
                TransmitErrorKind::Other => write!(f, "Other error"),
            }?;
            match self.message {
                Some(ref message) if !message.is_empty() => write!(f, ": {}", message)?,
                _ => {}
            }
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct TransmitError {
    kind: TransmitErrorKind,
    message: Option<String>,
}

impl TransmitError {
    pub fn kind(&self) -> TransmitErrorKind {
        self.kind.clone()
    }
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl std::error::Error for TransmitError {}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TransmitErrorKind {
    Timeout,
    LateCommand,
    BrokenChain,
    Overflow,
    OutOfSequence,
    Alignment,
    BadPacket,
    Other,
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
        // assert_eq!(false, metadata.out_of_sequence());
        // assert_eq!(false, metadata.more_fragments());
        // assert_eq!(0, metadata.fragment_offset());
        // assert!(metadata.last_error().is_none());
    }
}

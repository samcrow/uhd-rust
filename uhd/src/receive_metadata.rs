use std::ptr;

use crate::error::check_status;
use crate::utils::copy_string;
use crate::TimeSpec;

/// Data about a receive operation
pub struct ReceiveMetadata {
    /// Handle to C++ object
    handle: uhd_sys::uhd_rx_metadata_handle,
    /// Number of samples received
    samples: usize,
}

impl ReceiveMetadata {
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the timestamp of (the first?) of the received samples, according to the USRP's
    /// internal clock
    pub fn time_spec(&self) -> Option<TimeSpec> {
        if self.has_time_spec() {
            let mut time = TimeSpec::default();
            let mut seconds_time_t: libc::time_t = Default::default();

            check_status(unsafe {
                uhd_sys::uhd_rx_metadata_time_spec(
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
        check_status(unsafe { uhd_sys::uhd_rx_metadata_has_time_spec(self.handle, &mut has) })
            .unwrap();
        has
    }

    /// Returns true if the received samples are at the beginning of a burst
    pub fn start_of_burst(&self) -> bool {
        let mut value = false;
        check_status(unsafe { uhd_sys::uhd_rx_metadata_start_of_burst(self.handle, &mut value) })
            .unwrap();
        value
    }

    /// Returns true if the received samples are at the end of a burst
    pub fn end_of_burst(&self) -> bool {
        let mut value = false;
        check_status(unsafe { uhd_sys::uhd_rx_metadata_end_of_burst(self.handle, &mut value) })
            .unwrap();
        value
    }

    /// Returns true if the provided receive buffer was not large enough to hold a full packet
    ///
    /// If this is the case, the fragment_offset() function returns the offset from the beginning
    /// of the packet to the first sample received
    pub fn more_fragments(&self) -> bool {
        let mut value = false;
        check_status(unsafe { uhd_sys::uhd_rx_metadata_more_fragments(self.handle, &mut value) })
            .unwrap();
        value
    }

    /// If more_fragments() returned true, this function returns the offset from the beginning
    /// of the packet to the first sample received
    pub fn fragment_offset(&self) -> usize {
        let mut value = 0usize;
        check_status(unsafe {
            uhd_sys::uhd_rx_metadata_fragment_offset(
                self.handle,
                &mut value as *mut usize as *mut _,
            )
        })
        .unwrap();
        value
    }

    /// Returns true if a packet was dropped or received out of order
    pub fn out_of_sequence(&self) -> bool {
        let mut value = false;
        check_status(unsafe { uhd_sys::uhd_rx_metadata_out_of_sequence(self.handle, &mut value) })
            .unwrap();
        value
    }

    /// Returns the number of samples received
    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Sets the number of samples received
    pub(crate) fn set_samples(&mut self, samples: usize) {
        self.samples = samples
    }

    /// Returns the error code associated with the receive operation
    fn error_code(&self) -> uhd_sys::uhd_rx_metadata_error_code_t::Type {
        let mut code = uhd_sys::uhd_rx_metadata_error_code_t::UHD_RX_METADATA_ERROR_CODE_NONE;
        check_status(unsafe { uhd_sys::uhd_rx_metadata_error_code(self.handle, &mut code) })
            .unwrap();
        code
    }

    /// Returns the error associated with the receive operation, if any
    pub fn last_error(&self) -> Option<ReceiveError> {
        let out_of_sequence = self.out_of_sequence();
        use uhd_sys::uhd_rx_metadata_error_code_t::*;
        let kind = match self.error_code() {
            UHD_RX_METADATA_ERROR_CODE_TIMEOUT => ReceiveErrorKind::Timeout,
            UHD_RX_METADATA_ERROR_CODE_LATE_COMMAND => ReceiveErrorKind::LateCommand,
            UHD_RX_METADATA_ERROR_CODE_BROKEN_CHAIN => ReceiveErrorKind::BrokenChain,
            UHD_RX_METADATA_ERROR_CODE_OVERFLOW if !out_of_sequence => ReceiveErrorKind::Overflow,
            UHD_RX_METADATA_ERROR_CODE_OVERFLOW if out_of_sequence => {
                ReceiveErrorKind::OutOfSequence
            }
            UHD_RX_METADATA_ERROR_CODE_ALIGNMENT => ReceiveErrorKind::Alignment,
            UHD_RX_METADATA_ERROR_CODE_BAD_PACKET => ReceiveErrorKind::BadPacket,
            UHD_RX_METADATA_ERROR_CODE_NONE => {
                // Not actually an error
                return None;
            }
            _ => {
                // Some other error
                ReceiveErrorKind::Other
            }
        };
        let message = copy_string(|buffer, length| unsafe {
            uhd_sys::uhd_rx_metadata_strerror(self.handle, buffer, length as _)
        })
        .ok();

        Some(ReceiveError { kind, message })
    }

    pub(crate) fn handle_mut(&mut self) -> &mut uhd_sys::uhd_rx_metadata_handle {
        &mut self.handle
    }
}

// Thread safety: The uhd_rx_metadata struct just stores data. All exposed functions read fields.
unsafe impl Send for ReceiveMetadata {}
unsafe impl Sync for ReceiveMetadata {}

impl Default for ReceiveMetadata {
    fn default() -> Self {
        let mut handle: uhd_sys::uhd_rx_metadata_handle = ptr::null_mut();
        check_status(unsafe { uhd_sys::uhd_rx_metadata_make(&mut handle) }).unwrap();
        ReceiveMetadata { handle, samples: 0 }
    }
}

impl Drop for ReceiveMetadata {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_rx_metadata_free(&mut self.handle) };
    }
}

mod fmt {
    use super::{ReceiveError, ReceiveMetadata};
    use crate::ReceiveErrorKind;
    use std::fmt::{Debug, Display, Formatter, Result};

    impl Debug for ReceiveMetadata {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.debug_struct("ReceiveMetadata")
                .field("time_spec", &self.time_spec())
                .field("more_fragments", &self.more_fragments())
                .field("fragment_offset", &self.fragment_offset())
                .field("start_of_burst", &self.start_of_burst())
                .field("end_of_burst", &self.end_of_burst())
                .finish()
        }
    }

    impl Display for ReceiveError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self.kind {
                ReceiveErrorKind::Timeout => write!(f, "No packet received"),
                ReceiveErrorKind::LateCommand => write!(f, "Command timestamp was in the past"),
                ReceiveErrorKind::BrokenChain => write!(f, "Expected another stream command"),
                ReceiveErrorKind::Overflow => {
                    write!(f, "An internal receive buffer has been filled")
                }
                ReceiveErrorKind::OutOfSequence => write!(f, "Sequence error"),
                ReceiveErrorKind::Alignment => write!(f, "Multi-channel alignment failed"),
                ReceiveErrorKind::BadPacket => write!(f, "A packet could not be parsed"),
                ReceiveErrorKind::Other => write!(f, "Other error"),
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
pub struct ReceiveError {
    kind: ReceiveErrorKind,
    message: Option<String>,
}

impl ReceiveError {
    pub fn kind(&self) -> ReceiveErrorKind {
        self.kind.clone()
    }
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl std::error::Error for ReceiveError {}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ReceiveErrorKind {
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
    use super::ReceiveMetadata;

    #[test]
    fn default_rx_metadata() {
        let metadata = ReceiveMetadata::default();
        assert_eq!(None, metadata.time_spec());
        assert_eq!(false, metadata.start_of_burst());
        assert_eq!(false, metadata.end_of_burst());
        assert_eq!(false, metadata.out_of_sequence());
        assert_eq!(false, metadata.more_fragments());
        assert_eq!(0, metadata.fragment_offset());
        assert!(metadata.last_error().is_none());
    }
}

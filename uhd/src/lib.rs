//!
//! # `uhd`: Bindings to the USRP Hardware Driver library
//!
//! ## Status
//!
//! Basic functionality for configuring some USRP settings and receiving samples is working.
//!
//! Some things are not yet implemented:
//!
//! * Various configuration options related to transmitting
//! * Some configuration options related to receiving and time synchronization
//! * Sending samples to transmit
//!

extern crate libc;
extern crate num_complex;
extern crate uhd_sys;

use std::convert::TryInto;

mod daughter_board_eeprom;
mod error;
mod motherboard_eeprom;
pub mod range;
mod receiver;
mod stream;
mod string_vector;
mod transmitter;
mod tune_request;
mod tune_result;
mod usrp;
mod utils;

// Re-export many public items at the root
pub use daughter_board_eeprom::DaughterBoardEeprom;
pub use error::*;
pub use motherboard_eeprom::MotherboardEeprom;
pub use receiver::{error::*, info::ReceiveInfo, metadata::*, streamer::ReceiveStreamer};
pub use stream::*;
pub use transmitter::{info::TransmitInfo, metadata::*, streamer::TransmitStreamer};
pub use tune_request::*;
pub use tune_result::TuneResult;
pub use usrp::{SensorValue, Usrp};
pub use utils::alloc_boxed_slice;
// Common definitions

/// A UHD device time.
///
/// This type keeps UHD's split integer/floating-point representation contained in one value.
#[derive(Debug, Clone, Copy, Default, PartialOrd, PartialEq)]
pub struct TimeSpec {
    // In some versions of UHD, the corresponding field of uhd::time_spec_t is a time_t.
    // In other versions, it's a int64_t. The Rust code does conversion to keep this
    // an i64.
    pub seconds: i64,
    pub fraction: f64,
}

impl TimeSpec {
    /// Creates a device time from whole and fractional seconds.
    pub const fn new(full_secs: i64, frac_secs: f64) -> Self {
        Self {
            seconds: full_secs,
            fraction: frac_secs,
        }
    }

    /// Creates a device time from a total number of nanoseconds.
    ///
    /// Negative values are normalized so that the fractional component is in the range `[0, 1)`.
    pub fn from_nanos(nanoseconds: i64) -> Self {
        const NANOS_PER_SECOND: i64 = 1_000_000_000;

        let full_secs = nanoseconds.div_euclid(NANOS_PER_SECOND);
        let fractional_nanos = nanoseconds.rem_euclid(NANOS_PER_SECOND);
        Self::new(full_secs, fractional_nanos as f64 / NANOS_PER_SECOND as f64)
    }

    /// Converts this device time to a total number of nanoseconds.
    ///
    /// The fractional component is rounded to the nearest nanosecond.
    ///
    /// # Panics
    ///
    /// Panics if the fractional component is not finite or the result does not fit in an `i64`.
    pub fn into_nanos(self) -> i64 {
        const NANOS_PER_SECOND: i64 = 1_000_000_000;

        assert!(
            self.fraction.is_finite(),
            "TimeSpec fractional seconds must be finite"
        );
        let full_nanos = i128::from(self.seconds) * i128::from(NANOS_PER_SECOND);
        let fractional_nanos = (self.fraction * NANOS_PER_SECOND as f64).round() as i128;
        (full_nanos + fractional_nanos)
            .try_into()
            .expect("TimeSpec does not fit in i64 nanoseconds")
    }

    pub(crate) fn into_parts(self) -> (i64, f64) {
        (self.seconds, self.fraction)
    }
}

#[cfg(test)]
mod time_spec_tests {
    use super::TimeSpec;

    #[test]
    fn constructs_from_parts() {
        assert_eq!(
            TimeSpec::new(42, 0.25),
            TimeSpec {
                seconds: 42,
                fraction: 0.25,
            }
        );
    }

    #[test]
    fn constructs_from_nanoseconds() {
        assert_eq!(TimeSpec::from_nanos(1_500_000_000), TimeSpec::new(1, 0.5));
        assert_eq!(TimeSpec::from_nanos(-500_000_000), TimeSpec::new(-1, 0.5));
    }

    #[test]
    fn nanoseconds_round_trip() {
        for nanoseconds in [i64::MIN, -1_500_000_001, -1, 0, 1, 1_500_000_001, i64::MAX] {
            assert_eq!(TimeSpec::from_nanos(nanoseconds).into_nanos(), nanoseconds);
        }
    }
}

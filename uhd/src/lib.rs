extern crate uhd_sys;

mod daughter_board_eeprom;
mod error;
mod receive_metadata;
mod string_vector;
mod utils;

// Re-export all public items at the root
pub use crate::daughter_board_eeprom::DaughterBoardEeprom;
pub use crate::error::*;
pub use crate::receive_metadata::*;

// Common definitions

/// A time value, represented as an integer number of seconds and a floating-point fraction of
/// a second
#[derive(Debug, Clone, Default, PartialOrd, PartialEq)]
pub struct TimeSpec {
    pub seconds: i64,
    pub fraction: f64,
}

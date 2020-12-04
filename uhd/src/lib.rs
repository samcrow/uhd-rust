extern crate num_complex;
extern crate uhd_sys;

mod daughter_board_eeprom;
mod error;
mod motherboard_eeprom;
pub mod range;
mod receive_info;
mod receive_metadata;
mod receive_streamer;
mod stream;
mod string_vector;
mod tune_request;
mod tune_result;
mod usrp;
mod utils;

// Re-export many public items at the root
pub use crate::daughter_board_eeprom::DaughterBoardEeprom;
pub use crate::error::*;
pub use crate::motherboard_eeprom::MotherboardEeprom;
pub use crate::receive_info::ReceiveInfo;
pub use crate::receive_metadata::*;
pub use crate::receive_streamer::ReceiveStreamer;
pub use crate::stream::*;
pub use crate::tune_request::*;
pub use crate::tune_result::TuneResult;
pub use crate::usrp::Usrp;

// Common definitions

/// A time value, represented as an integer number of seconds and a floating-point fraction of
/// a second
#[derive(Debug, Clone, Default, PartialOrd, PartialEq)]
pub struct TimeSpec {
    pub seconds: i64,
    pub fraction: f64,
}

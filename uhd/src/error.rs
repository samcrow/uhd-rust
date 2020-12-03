use crate::utils::copy_string;
use std::ffi::NulError;
use std::str::Utf8Error;

/// Result type alias
pub type Result<T> = ::std::result::Result<T, Error>;

/// A UHD error
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            message: None,
        }
    }
    /// Creates an error with the provided kind, and a message from uhd_get_last_error()
    ///
    /// Returns None if code is UHD_ERROR_NONE
    pub(crate) fn from_code_with_last_error(code: uhd_sys::uhd_error::Type) -> Option<Error> {
        let kind = code_to_error_kind(code)?;
        Some(Error {
            kind,
            message: last_error_message(),
        })
    }

    /// Returns the type of this error
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
    /// Returns a description of this error, if available
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// Returns a string copied using uhd_get_last_error()
fn last_error_message() -> Option<String> {
    copy_string(|buffer, length| unsafe { uhd_sys::uhd_get_last_error(buffer, length as _) }).ok()
}

/// Converts a status code into a result
pub(crate) fn check_status(status: uhd_sys::uhd_error::Type) -> std::result::Result<(), Error> {
    if let Some(e) = Error::from_code_with_last_error(status) {
        Err(e)
    } else {
        Ok(())
    }
}

/// Kinds of errors
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ErrorKind {
    InvalidDevice,
    Index,
    Key,
    NotImplemented,
    Usb,
    Io,
    Os,
    Assertion,
    Lookup,
    Type,
    Value,
    Runtime,
    Environment,
    System,
    Except,
    BoostExcept,
    StdExcept,
    /// A string containing a null byte was provided
    NullByte,
    /// A string was too long to be copied out through C FFI
    StringLength,
    /// A string from C FFI contained invalid UTF-8
    Utf8,
    Unknown,
}

impl std::error::Error for Error {}

fn code_to_error_kind(code: uhd_sys::uhd_error::Type) -> Option<ErrorKind> {
    use uhd_sys::uhd_error;
    match code {
        uhd_error::UHD_ERROR_NONE => None,
        uhd_error::UHD_ERROR_INVALID_DEVICE => Some(ErrorKind::InvalidDevice),
        uhd_error::UHD_ERROR_INDEX => Some(ErrorKind::Index),
        uhd_error::UHD_ERROR_KEY => Some(ErrorKind::Key),
        uhd_error::UHD_ERROR_NOT_IMPLEMENTED => Some(ErrorKind::NotImplemented),
        uhd_error::UHD_ERROR_USB => Some(ErrorKind::Usb),
        uhd_error::UHD_ERROR_IO => Some(ErrorKind::Io),
        uhd_error::UHD_ERROR_OS => Some(ErrorKind::Os),
        uhd_error::UHD_ERROR_ASSERTION => Some(ErrorKind::Assertion),
        uhd_error::UHD_ERROR_LOOKUP => Some(ErrorKind::Lookup),
        uhd_error::UHD_ERROR_TYPE => Some(ErrorKind::Type),
        uhd_error::UHD_ERROR_VALUE => Some(ErrorKind::Value),
        uhd_error::UHD_ERROR_RUNTIME => Some(ErrorKind::Runtime),
        uhd_error::UHD_ERROR_ENVIRONMENT => Some(ErrorKind::Environment),
        uhd_error::UHD_ERROR_SYSTEM => Some(ErrorKind::System),
        uhd_error::UHD_ERROR_EXCEPT => Some(ErrorKind::Except),
        uhd_error::UHD_ERROR_BOOSTEXCEPT => Some(ErrorKind::BoostExcept),
        uhd_error::UHD_ERROR_STDEXCEPT => Some(ErrorKind::StdExcept),
        uhd_error::UHD_ERROR_UNKNOWN | _ => Some(ErrorKind::Unknown),
    }
}

impl From<NulError> for Error {
    fn from(_: NulError) -> Self {
        Error::new(ErrorKind::NullByte)
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::new(ErrorKind::Utf8)
    }
}

mod fmt {
    use super::{Error, ErrorKind};
    use std::fmt::{Display, Formatter, Result};
    impl Display for Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            if let Some(ref message) = self.message {
                // If a message is available, use it
                write!(f, "{}", message)
            } else {
                // Otherwise, provide a generic message based on the kind
                match self.kind {
                    ErrorKind::InvalidDevice => write!(f, "Invalid device arguments"),
                    ErrorKind::Index => {
                        write!(f, "uhd::index_error - A sequence index is out of range")
                    }
                    ErrorKind::Key => write!(f, "uhd::key_error - Invalid key"),
                    ErrorKind::NotImplemented => {
                        write!(f, "uhd::not_implemented_error - Not implemented")
                    }
                    ErrorKind::Usb => write!(f, "uhd::usb_error - USB communication problem"),
                    ErrorKind::Io => write!(f, "uhd::io_error - Input/output error"),
                    ErrorKind::Os => write!(f, "uhd::os_error - System-related error"),
                    ErrorKind::Assertion => write!(f, "uhd::assertion_error - Assertion failed"),
                    ErrorKind::Lookup => write!(f, "uhd::lookup_error - Invalid index or key"),
                    ErrorKind::Type => write!(f, "uhd::type_error - Value has incorrect type"),
                    ErrorKind::Value => write!(f, "uhd::value_error - Invalid value"),
                    ErrorKind::Runtime => write!(f, "uhd::runtime_error - Other runtime error"),
                    ErrorKind::Environment => {
                        write!(f, "uhd::environment_error - Environment error")
                    }
                    ErrorKind::System => write!(f, "uhd::system_error - System-related error"),
                    ErrorKind::Except => write!(f, "uhd::exception - Other UHD exception"),
                    ErrorKind::BoostExcept => write!(f, "A boost::exception was thrown"),
                    ErrorKind::StdExcept => write!(f, "A std::exception was thrown"),
                    ErrorKind::NullByte => write!(f, "Null byte in input string"),
                    ErrorKind::StringLength => write!(f, "String from FFI is too long"),
                    ErrorKind::Utf8 => write!(f, "String from FFI contains invalid UTF-8"),
                    ErrorKind::Unknown => write!(f, "Unknown error"),
                }
            }
        }
    }
}

use crate::error::{check_status, Error, ErrorKind};
use crate::utils::copy_string;
use std::ffi::CString;
use std::ptr;

/// Information stored in the USRP motherboard EEPROM
pub struct MotherboardEeprom(uhd_sys::uhd_mboard_eeprom_handle);

impl MotherboardEeprom {
    pub fn get(&self, key: &str) -> Result<Option<String>, Error> {
        let key = CString::new(key)?;
        let status = copy_string(|buffer, length| unsafe {
            uhd_sys::uhd_mboard_eeprom_get_value(self.0, key.as_ptr(), buffer, length as _)
        });
        // An error with kind Key indicates that the value was not found
        match status {
            Ok(value) => Ok(Some(value)),
            Err(e) => match e.kind() {
                ErrorKind::Key => Ok(None),
                _ => Err(e),
            },
        }
    }

    pub fn put(&mut self, key: String, value: String) -> Result<(), Error> {
        let key = CString::new(key)?;
        let value = CString::new(value)?;
        check_status(unsafe {
            uhd_sys::uhd_mboard_eeprom_set_value(self.0, key.as_ptr(), value.as_ptr())
        })
    }

    pub(crate) fn handle(&mut self) -> uhd_sys::uhd_mboard_eeprom_handle {
        self.0
    }
}

impl Default for MotherboardEeprom {
    fn default() -> Self {
        let mut handle = ptr::null_mut();
        check_status(unsafe { uhd_sys::uhd_mboard_eeprom_make(&mut handle) }).unwrap();
        MotherboardEeprom(handle)
    }
}

impl Drop for MotherboardEeprom {
    fn drop(&mut self) {
        let _ = unsafe { uhd_sys::uhd_mboard_eeprom_free(&mut self.0) };
    }
}

#[cfg(test)]
mod test {
    use super::MotherboardEeprom;

    #[test]
    fn empty_eeprom() {
        let eeprom = MotherboardEeprom::default();
        assert_eq!(Ok(None), eeprom.get("jabberwock".into()));
    }
}

use crate::{encode_report, encode_reset_report, Device, DeviceError};
use keysteel_core::frame::Frame;

pub const VENDOR_ID: u16 = 0x1038;
pub const PRODUCT_ID: u16 = 0x1642;
/// The OLED is driven through this HID interface.
pub const INTERFACE: i32 = 1;

/// True for the kernel's `HID_ID` value (`bus:vendor:product`, each in hex)
/// of the Apex Pro TKL Gen 3.
pub fn is_keyboard_hid_id(hid_id: &str) -> bool {
    let mut parts = hid_id.split(':').skip(1).map(|p| u32::from_str_radix(p, 16));
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(Ok(vendor)), Some(Ok(product)), None)
            if vendor == VENDOR_ID as u32 && product == PRODUCT_ID as u32
    )
}

pub struct HidrawDevice {
    handle: hidapi::HidDevice,
}

impl HidrawDevice {
    /// Opens the display interface of the Apex Pro TKL Gen 3.
    pub fn open() -> Result<HidrawDevice, DeviceError> {
        let api = hidapi::HidApi::new().map_err(|e| DeviceError::Hid(e.to_string()))?;
        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VENDOR_ID
                    && d.product_id() == PRODUCT_ID
                    && d.interface_number() == INTERFACE
            })
            .ok_or(DeviceError::NotFound)?;
        let handle = info
            .open_device(&api)
            .map_err(|e| DeviceError::Hid(e.to_string()))?;
        Ok(HidrawDevice { handle })
    }
}

impl Device for HidrawDevice {
    fn send_frame(&mut self, frame: &Frame) -> Result<(), DeviceError> {
        self.handle
            .send_feature_report(&encode_report(frame))
            .map_err(|e| DeviceError::Hid(e.to_string()))
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.handle
            .write(&encode_reset_report())
            .map(|_| ())
            .map_err(|e| DeviceError::Hid(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hid_id_of_the_keyboard_matches() {
        assert!(is_keyboard_hid_id("0003:00001038:00001642"));
    }

    #[test]
    fn hid_id_of_another_product_does_not_match() {
        assert!(!is_keyboard_hid_id("0003:00001038:00001618"));
        assert!(!is_keyboard_hid_id("0003:0000046D:0000C52B"));
    }

    #[test]
    fn garbage_hid_id_does_not_match() {
        assert!(!is_keyboard_hid_id(""));
        assert!(!is_keyboard_hid_id("1038:1642"));
        assert!(!is_keyboard_hid_id("0003:zzzz:yyyy"));
    }

    #[test]
    fn hid_id_of_another_vendor_with_the_same_product_does_not_match() {
        assert!(!is_keyboard_hid_id("0003:0000103a:00001642"));
    }
}

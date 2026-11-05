use crate::{encode_report, Device, DeviceError};
use keysteel_core::frame::Frame;

pub const VENDOR_ID: u16 = 0x1038;
pub const PRODUCT_ID: u16 = 0x1642;
/// The OLED is driven through this HID interface.
pub const INTERFACE: i32 = 1;

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
}

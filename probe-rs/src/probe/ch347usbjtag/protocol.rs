use nusb::{DeviceInfo, Interface};

use crate::probe::{self, DebugProbeInfo, DebugProbeSelector, ProbeCreationError};

use super::Ch347UsbJtagFactory;

const CH34X_VID_PID: [(u16, u16); 3] = [(0x1A86, 0x55DE), (0x1A86, 0x55DD), (0x1A86, 0x55E8)];

pub(crate) fn is_ch34x_device(device: &DeviceInfo) -> bool {
    CH34X_VID_PID.contains(&(device.vendor_id(), device.product_id()))
}

#[derive(Debug)]
enum PACK {
    STANDARD_PACK,
    LARGER_PACK,
}

pub struct Ch347UsbJtagDevice {
    device: Interface,
    name: String,
    epout: u8,
    epin: u8,
    pack: Option<PACK>,
}

impl std::fmt::Debug for Ch347UsbJtagDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ch347UsbJtagDevice")
            .field("name", &self.name)
            .field("epout", &self.epout)
            .field("epin", &self.epin)
            .field("pack", &self.pack)
            .finish()
    }
}

impl Ch347UsbJtagDevice {
    pub fn new_from_selector(selector: &DebugProbeSelector) -> Result<Self, ProbeCreationError> {
        let device = nusb::list_devices()
            .map_err(ProbeCreationError::Usb)?
            .filter(is_ch34x_device)
            .next()
            .ok_or(ProbeCreationError::NotFound)?;

        // tracing::debug!("{:?}", device);

        let device_handle = device.open().map_err(probe::ProbeCreationError::Usb)?;

        let config = device_handle
            .configurations()
            .next()
            .expect("Can get usb device configs");

        tracing::info!("Active config descriptor: {:?}", config);

        for interface in config.interfaces() {
            let interface_number = interface.interface_number();

            let Some(descriptor) = interface.alt_settings().next() else {
                continue;
            };

            if (!(descriptor.class() != 255
                && descriptor.subclass() != 0
                && descriptor.protocol() != 0))
            {
                continue;
            }
        }

        // if should use config to get current interface, this only work in ch347f
        let interface = device_handle
            .claim_interface(4)
            .map_err(ProbeCreationError::Usb)?;

        Ok(Self {
            device: interface,
            name: "ch347".into(),
            epout: 0x06,
            epin: 0x86,
            pack: None,
        })
    }
}

pub(super) fn list_ch347usbjtag_devices() -> Vec<DebugProbeInfo> {
    let Ok(devices) = nusb::list_devices() else {
        return vec![];
    };

    devices
        .filter(is_ch34x_device)
        .map(|device| {
            DebugProbeInfo::new(
                "CH347 USB Jtag".to_string(),
                device.vendor_id(),
                device.product_id(),
                device.serial_number().map(Into::into),
                &Ch347UsbJtagFactory,
                None,
            )
        })
        .collect()
}

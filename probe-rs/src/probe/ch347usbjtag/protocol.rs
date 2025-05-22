use std::time::Duration;

use bitvec::vec::BitVec;
use nusb::{DeviceInfo, Interface};

use crate::probe::{
    self, DebugProbeError, DebugProbeInfo, DebugProbeSelector, ProbeCreationError, RawJtagIo,
    usb_util::InterfaceExt,
};

use super::Ch347UsbJtagFactory;

const CH34X_VID_PID: [(u16, u16); 3] = [(0x1A86, 0x55DE), (0x1A86, 0x55DD), (0x1A86, 0x55E8)];
const REV_BUF_MAX: usize = 64;

pub(crate) fn is_ch34x_device(device: &DeviceInfo) -> bool {
    CH34X_VID_PID.contains(&(device.vendor_id(), device.product_id()))
}

#[derive(Debug)]
enum PACK {
    STANDARD_PACK,
    LARGER_PACK,
}

enum Command {
    JTAG_INIT,
    JTAG_BIT_OP,
    JTAG_BIT_OP_RD,
    JTAG_DATA_SHIFT,
    JTAG_DATA_SHIFT_RD,
}

struct Clock {
    tms: bool,
    tdi: bool,
    trst: bool,
}

impl From<Clock> for u8 {
    fn from(value: Clock) -> Self {
        let Clock { tms, tdi, trst } = value;
        0 | u8::from(tms) << 1 | u8::from(tdi) << 4 | u8::from(trst) << 5
    }
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        match value {
            Command::JTAG_INIT => 0xD0,
            Command::JTAG_BIT_OP => 0xD1,
            Command::JTAG_BIT_OP_RD => 0xD2,
            Command::JTAG_DATA_SHIFT => 0xD3,
            Command::JTAG_DATA_SHIFT_RD => 0xD4,
        }
    }
}

pub struct Ch347UsbJtagDevice {
    device: Interface,
    name: String,
    comand_quene: Option<Command>,
    output_buffer: Vec<u8>,
    response: BitVec,
    epout: u8,
    epin: u8,
    pack: Option<PACK>,
    speed_khz: u32,
}

impl std::fmt::Debug for Ch347UsbJtagDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ch347UsbJtagDevice")
            .field("name", &self.name)
            .field("epout", &self.epout)
            .field("epin", &self.epin)
            .field("pack", &self.pack)
            .field("speed", &self.speed_khz)
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
            comand_quene: None,
            output_buffer: Vec::new(),
            response: BitVec::new(),
            epout: 0x06,
            epin: 0x86,
            pack: None,
            speed_khz: 15000,
        })
    }

    pub fn attach(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    fn speed_khz(&self) -> u32 {
        self.speed_khz
    }

    fn set_speed_khz(&mut self, speed_khz: u32) -> u32 {
        self.speed_khz = speed_khz;
        self.speed_khz
    }

    fn pack(&self) -> &Option<PACK> {
        &self.pack
    }

    fn apply_clock_speed(&mut self, speed_khz: u32) -> Result<u32, DebugProbeError> {
        let index;
        let mut buf = [0; 4];
        match *self.pack() {
            Some(PACK::STANDARD_PACK) => {
                index = match speed_khz {
                    1875 => 0,
                    3750 => 1,
                    7500 => 2,
                    15000 => 3,
                    30000 => 4,
                    60000 => 5,
                    _ => return Err(DebugProbeError::UnsupportedSpeed(speed_khz)),
                };
            }
            Some(PACK::LARGER_PACK) => {
                index = match speed_khz {
                    468 => 0,
                    937 => 1,
                    1875 => 2,
                    3750 => 3,
                    7500 => 4,
                    15000 => 5,
                    30000 => 6,
                    60000 => 7,
                    _ => return Err(DebugProbeError::UnsupportedSpeed(speed_khz)),
                }
            }
            None => return Err(DebugProbeError::UnsupportedSpeed(speed_khz)),
        }
        self.device
            .write_bulk(
                self.epout,
                &[0xD0, 0x06, 0x00, 0x00, index, 0x00, 0x00, 0x00, 0x00],
                Duration::from_millis(500),
            )
            .map_err(DebugProbeError::Usb)?;

        self.device
            .read_bulk(self.epin, &mut buf, Duration::from_millis(500))
            .map_err(DebugProbeError::Usb)?;
        if buf[3] == 0x00 {
            Ok(speed_khz)
        } else {
            Err(DebugProbeError::UnsupportedSpeed(speed_khz))
        }
    }

    fn read_response(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    fn append_command(&mut self, command: Command) -> Result<(), DebugProbeError> {
        Ok(())
    }

    fn finalize_command(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    pub(crate) fn shift_bit(
        &mut self,
        tms: bool,
        tdi: bool,
        capture: bool,
    ) -> Result<(), DebugProbeError> {
        Ok(())
    }

    pub(crate) fn send_buffer(&mut self) -> Result<(), DebugProbeError> {
        Ok(())
    }

    pub(crate) fn read_captured_bits(&mut self) -> Result<BitVec, DebugProbeError> {
        todo!()
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

//! ch347 is a usb bus converter that provides UART, I2C and SPI and Jtag/Swd interface
mod protocol;

use protocol::Ch347UsbJtagDevice;

use crate::probe::{DebugProbe, ProbeFactory};

#[derive(Debug)]
pub struct Ch347UsbJtagFactory;

impl std::fmt::Display for Ch347UsbJtagFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ch347UsbJtag")
    }
}

#[derive(Debug)]
pub struct Ch347UsbJtag {
    device: Ch347UsbJtagDevice,
}

impl ProbeFactory for Ch347UsbJtagFactory {
    fn open(
        &self,
        selector: &super::DebugProbeSelector,
    ) -> Result<Box<dyn super::DebugProbe>, super::DebugProbeError> {
        let ch347 = Ch347UsbJtagDevice::new_from_selector(selector)?;

        Ok(Box::new(Ch347UsbJtag { device: ch347 }))
    }

    fn list_probes(&self) -> Vec<super::DebugProbeInfo> {
        protocol::list_ch347usbjtag_devices()
    }
}

impl DebugProbe for Ch347UsbJtag {
    fn get_name(&self) -> &str {
        "CH347 USB Jtag"
    }

    fn speed_khz(&self) -> u32 {
        // TODO
        30000
    }

    fn set_speed(&mut self, speed_khz: u32) -> Result<u32, super::DebugProbeError> {
        // TODO
        Ok(speed_khz)
    }

    fn attach(&mut self) -> Result<(), super::DebugProbeError> {
        // TODO
        Ok(())
    }

    fn detach(&mut self) -> Result<(), crate::Error> {
        // TODO
        Ok(())
    }

    fn target_reset(&mut self) -> Result<(), super::DebugProbeError> {
        // TODO
        Ok(())
    }
    fn target_reset_assert(&mut self) -> Result<(), super::DebugProbeError> {
        // TODO
        Ok(())
    }

    fn target_reset_deassert(&mut self) -> Result<(), super::DebugProbeError> {
        // TODO
        Ok(())
    }

    fn select_protocol(
        &mut self,
        protocol: super::WireProtocol,
    ) -> Result<(), super::DebugProbeError> {
        // TODO
        Ok(())
    }

    fn active_protocol(&self) -> Option<super::WireProtocol> {
        // TODO
        Some(super::WireProtocol::Jtag)
    }

    fn into_probe(self: Box<Self>) -> Box<dyn DebugProbe> {
        self
    }
}

use std::{thread, time::Duration};

use anyhow::{bail, Result};
use objc::runtime::Object;
use crate::LuminancePacket;

#[repr(C)]
pub struct IOAVService(Object);

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOAVServiceCreate(allocator: core_foundation::base::CFAllocatorRef) -> *mut IOAVService;

    fn IOAVServiceWriteI2C(
        service: *mut IOAVService,
        dev_addr: u8,
        input_addr: u8,
        data: *const u8,
        size: usize,
    ) -> i32;
}

pub struct DisplayService {
    service: *mut IOAVService,
}

impl DisplayService {
    pub fn try_new() -> Result<Self> {
        let service = unsafe { IOAVServiceCreate(std::ptr::null()) };
        if service.is_null() {
            bail!("Failed to create IOAVService");
        }
        Ok(DisplayService { service })
    }

    pub fn set_luminance(&self, packet: &LuminancePacket) -> Result<()> {
        // Find the number of bytes used in the packet
        let bytes_used = packet
            .data
            .iter()
            .rposition(|&x| x != 0)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // Wait a bit before writing the I2C data
        thread::sleep(Duration::from_millis(50));

        let result = unsafe {
            IOAVServiceWriteI2C(
                self.service,
                0x37,
                packet.input_addr,
                packet.data.as_ptr(),
                bytes_used,
            )
        };

        match result {
            0 => Ok(()),
            _ => bail!("Failed to write I2C data: {result}"),
        }
    }
}

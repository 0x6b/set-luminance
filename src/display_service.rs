use std::{ptr::null, thread::sleep, time::Duration};

use anyhow::{bail, Result};
use core_foundation::base::CFAllocatorRef;
use log::debug;
use objc::runtime::Object;

use crate::LuminancePacket;

const DEFAULT_INPUT_ADDRESS: u8 = 0x6e;

#[repr(C)]
pub struct IOAVService(Object);

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOAVServiceCreate(allocator: CFAllocatorRef) -> *mut IOAVService;

    fn IOAVServiceWriteI2C(
        service: *mut IOAVService,
        dev_addr: u8,
        input_addr: u8,
        data: *const u8,
        size: usize,
    ) -> i32;

    fn IOAVServiceReadI2C(
        service: *mut IOAVService,
        dev_addr: u8,
        input_addr: u8,
        data: *mut u8,
        size: usize,
    ) -> i32;
}

pub struct DisplayService {
    service: *mut IOAVService,
}

impl DisplayService {
    pub fn try_new() -> Result<Self> {
        let service = unsafe { IOAVServiceCreate(null()) };
        if service.is_null() {
            bail!("Failed to create IOAVService");
        }
        Ok(DisplayService { service })
    }

    pub fn set_luminance(&self, value: u8) -> Result<()> {
        self.send_packet(&LuminancePacket::new_write_packet(value))
    }

    pub fn get_luminance(&self) -> Result<u8> {
        self.send_packet(&LuminancePacket::new_read_packet())?;
        sleep(Duration::from_millis(50));
        let response = self.receive_packet()?;
        Ok(response[9])
    }

    pub fn send_packet(&self, packet: &LuminancePacket) -> Result<()> {
        debug!("Packet to send: {packet:?}");

        // Find the number of bytes used in the packet
        let bytes_used = packet
            .data
            .iter()
            .rposition(|&x| x != 0)
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // Wait a bit before writing the I2C data
        sleep(Duration::from_millis(50));

        let result = unsafe {
            IOAVServiceWriteI2C(
                self.service,
                0x37,
                DEFAULT_INPUT_ADDRESS,
                packet.data.as_ptr(),
                bytes_used,
            )
        };

        match result {
            0 => Ok(()),
            _ => bail!("Failed to write I2C data: {result}"),
        }
    }

    pub fn receive_packet(&self) -> Result<Vec<u8>> {
        let mut response = [0u8; 16];
        let result = unsafe {
            IOAVServiceReadI2C(
                self.service,
                0x37,
                DEFAULT_INPUT_ADDRESS,
                response.as_mut_ptr(),
                16, // Increased size to ensure we get all necessary bytes
            )
        };

        debug!("Packet received: {response:02X?}");

        if result != 0 {
            bail!("Failed to read I2C data: {result}");
        }

        if response[0] != 0x6E {
            bail!("Unexpected response format");
        }

        Ok(response.to_vec())
    }
}

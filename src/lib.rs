use std::{
    ffi::c_void,
    ptr::{self, NonNull},
    thread::sleep,
    time::Duration,
};

use ptr::null;

/// DDC/CI sink address used by the display.
const CHIP_ADDR: u32 = 0x37;
/// I2C input address for DDC traffic.
const INPUT_ADDR: u32 = 0x51;
/// Delay between DDC transactions to give the monitor time to respond.
const DDC_WAIT: Duration = Duration::from_micros(10_000);
/// Base checksum seed prescribed by the DDC/CI spec.
const CHECKSUM_SEED: u8 = 0x6e;
/// DDC/CI "get VCP feature" header for luminance (VCP code 0x10).
const READ_HEADER: [u8; 3] = [0x82, 0x01, 0x10];

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOAVServiceCreate(allocator: *const c_void) -> *mut c_void;
    fn IOAVServiceWriteI2C(
        service: *mut c_void,
        chip_addr: u32,
        input_addr: u32,
        data: *const u8,
        len: u32,
    ) -> i32;
    fn IOAVServiceReadI2C(
        service: *mut c_void,
        chip_addr: u32,
        input_addr: u32,
        data: *mut u8,
        len: u32,
    ) -> i32;
}

/// Return the current luminance as reported by the display.
pub fn current_luminance() -> Result<u16, String> {
    let service =
        Ddc::connect().ok_or_else(|| "Could not find a suitable external display.".to_string())?;
    service.luminance().map_err(|err| format!("DDC read failed: {err}"))
}

/// Set the luminance level on the display (clamp upstream if needed).
pub fn set_luminance(value: u8) -> Result<(), String> {
    let value = value.min(100);
    let service =
        Ddc::connect().ok_or_else(|| "Could not find a suitable external display.".to_string())?;
    service
        .set_luminance(value.into())
        .map_err(|err| format!("DDC write failed: {err}"))
}

struct Ddc(NonNull<c_void>);

impl Ddc {
    fn connect() -> Option<Self> {
        NonNull::new(unsafe { IOAVServiceCreate(null()) }).map(Self)
    }

    fn write(&self, payload: &[u8]) -> Result<(), i32> {
        sleep(DDC_WAIT);
        let ret = unsafe {
            IOAVServiceWriteI2C(
                self.0.as_ptr(),
                CHIP_ADDR,
                INPUT_ADDR,
                payload.as_ptr(),
                payload.len() as u32,
            )
        };
        if ret == 0 { Ok(()) } else { Err(ret) }
    }

    fn read(&self, payload: &[u8], buffer: &mut [u8]) -> Result<(), i32> {
        self.write(payload)?;
        sleep(DDC_WAIT);
        let ret = unsafe {
            IOAVServiceReadI2C(
                self.0.as_ptr(),
                CHIP_ADDR,
                INPUT_ADDR,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            )
        };
        if ret == 0 { Ok(()) } else { Err(ret) }
    }

    fn luminance(&self) -> Result<u16, i32> {
        let mut request = [0u8; 4];
        request[..3].copy_from_slice(&READ_HEADER);
        request[3] = checksum(CHECKSUM_SEED, &request[..3]);

        let mut response = [0u8; 12];
        self.read(&request, &mut response)?;
        Ok(u16::from_be_bytes([response[8], response[9]]))
    }

    fn set_luminance(&self, value: u8) -> Result<(), i32> {
        let [hi, lo] = (value as u16).to_be_bytes();
        let mut payload = [0x84, 0x03, 0x10, hi, lo, 0];
        payload[5] = checksum(CHECKSUM_SEED ^ INPUT_ADDR as u8, &payload[..5]);

        for _ in 0..2 {
            self.write(&payload)?;
        }
        Ok(())
    }
}

fn checksum(seed: u8, bytes: &[u8]) -> u8 {
    bytes.iter().fold(seed, |acc, &b| acc ^ b)
}

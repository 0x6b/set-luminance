use std::{
    ffi::c_void,
    ptr::{NonNull, null},
    thread::sleep,
    time::Duration,
};

/// DDC/CI sink address used by the display.
const CHIP_ADDR: u32 = 0x37;
/// I2C input address for DDC traffic.
const INPUT_ADDR: u32 = 0x51;
/// Delay between DDC transactions to give the monitor time to respond.
const DDC_WAIT: Duration = Duration::from_micros(10_000);
/// Number of retries for read operations when a malformed frame is received.
const READ_RETRIES: usize = 3;
/// Base checksum seed prescribed by the DDC/CI spec.
const CHECKSUM_SEED: u8 = 0x6e;
/// Base checksum seed for monitor -> host responses.
const RESPONSE_CHECKSUM_SEED: u8 = (((CHIP_ADDR as u8) << 1) | 0x01) ^ INPUT_ADDR as u8;
/// DDC/CI "get VCP feature" response opcode.
const READ_RESPONSE_OPCODE: u8 = 0x02;
/// DDC/CI "get VCP feature" header for luminance (VCP code 0x10).
const READ_HEADER: [u8; 3] = [0x82, 0x01, 0x10];
/// Result code for successful VCP reads.
const VCP_OK: u8 = 0x00;
/// Error code used when monitor I2C responds with malformed data.
const ERR_INVALID_RESPONSE: i32 = -1;

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
pub fn current_luminance() -> Result<u8, String> {
    connect()?
        .luminance()
        .map_err(|err| format!("DDC read failed: {err}"))
}

/// Set the luminance level on the display (clamp upstream if needed).
pub fn set_luminance(value: u8) -> Result<(), String> {
    connect()?
        .set_luminance(value.min(100))
        .map_err(|err| format!("DDC write failed: {err}"))
}

fn connect() -> Result<Ddc, String> {
    Ddc::connect().ok_or_else(|| "Could not find a suitable external display.".to_string())
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

    fn luminance(&self) -> Result<u8, i32> {
        let (current, max) = self.read_luminance_feature()?;
        Ok(normalize_percent(current, max))
    }

    fn read_luminance_feature(&self) -> Result<(u16, u16), i32> {
        let mut request = [0u8; 4];
        request[..3].copy_from_slice(&READ_HEADER);
        request[3] = checksum(CHECKSUM_SEED, &request[..3]);

        let mut last_error = ERR_INVALID_RESPONSE;
        for _ in 0..READ_RETRIES {
            let mut frame = [0u8; 12];
            match self.read(&request, &mut frame).and_then(|_| {
                parse_vcp_feature_response(&frame, READ_HEADER[2]).ok_or(ERR_INVALID_RESPONSE)
            }) {
                Ok(value) => return Ok(value),
                Err(err) => last_error = err,
            }
        }
        Err(last_error)
    }

    fn set_luminance(&self, value: u8) -> Result<(), i32> {
        let [hi, lo] = (value as u16).to_be_bytes();
        let mut payload = [0x84, 0x03, 0x10, hi, lo, 0];
        payload[5] = checksum(CHECKSUM_SEED ^ INPUT_ADDR as u8, &payload[..5]);

        (0..2).try_for_each(|_| self.write(&payload))
    }
}

fn checksum(seed: u8, bytes: &[u8]) -> u8 {
    bytes.iter().fold(seed, |acc, &b| acc ^ b)
}

fn parse_vcp_feature_response(frame: &[u8], feature: u8) -> Option<(u16, u16)> {
    if frame.len() < 11 {
        return None;
    }

    let payload_len = (frame[1] & 0x7f) as usize;
    let payload_end = 2 + payload_len;
    if payload_len < 8 || payload_end >= frame.len() {
        return None;
    }

    let payload = &frame[2..payload_end];
    if frame[payload_end] != checksum(RESPONSE_CHECKSUM_SEED, &frame[1..payload_end]) {
        return None;
    }

    if payload[0] != READ_RESPONSE_OPCODE || payload[1] != VCP_OK || payload[2] != feature {
        return None;
    }

    let max = u16::from_be_bytes([payload[4], payload[5]]);
    let current = u16::from_be_bytes([payload[6], payload[7]]);
    (max != 0).then_some((current.min(max), max))
}

fn normalize_percent(current: u16, max: u16) -> u8 {
    if max == 0 {
        return current.min(100) as u8;
    }

    // DDC/CI reports raw current/max values; convert them to a rounded 0-100 percentage.
    let current = current.min(max);
    let percent = (u32::from(current) * 100 + u32::from(max) / 2) / u32::from(max);
    percent.min(100) as u8
}

#[cfg(test)]
mod tests {
    use super::{
        CHECKSUM_SEED, RESPONSE_CHECKSUM_SEED, normalize_percent, parse_vcp_feature_response,
    };

    #[test]
    fn normalize_percent_rounds_to_nearest() {
        assert_eq!(normalize_percent(70, 100), 70);
        assert_eq!(normalize_percent(37, 53), 70);
        assert_eq!(normalize_percent(1, 3), 33);
    }

    #[test]
    fn parses_valid_get_vcp_response() {
        let mut frame = [0u8; 12];
        frame[0] = CHECKSUM_SEED;
        frame[1] = 0x88;
        frame[2] = 0x02;
        frame[3] = 0x00;
        frame[4] = 0x10;
        frame[5] = 0x00;
        frame[6] = 0x00;
        frame[7] = 0x64;
        frame[8] = 0x00;
        frame[9] = 0x46;
        frame[10] = super::checksum(RESPONSE_CHECKSUM_SEED, &frame[1..10]);

        assert_eq!(parse_vcp_feature_response(&frame, 0x10), Some((70, 100)));
    }

    #[test]
    fn rejects_invalid_checksum() {
        let mut frame = [0u8; 12];
        frame[1] = 0x88;
        frame[2] = 0x02;
        frame[3] = 0x00;
        frame[4] = 0x10;
        frame[5] = 0x00;
        frame[6] = 0x00;
        frame[7] = 0x64;
        frame[8] = 0x00;
        frame[9] = 0x46;
        frame[10] = 0xff;

        assert_eq!(parse_vcp_feature_response(&frame, 0x10), None);
    }
}

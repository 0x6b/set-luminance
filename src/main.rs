use std::{env, ffi::c_void, process::exit, ptr::null, thread::sleep, time::Duration};

const INPUT_ADDR: u8 = 0x51;
const DDC_WAIT: Duration = Duration::from_micros(10000);

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

fn get_luminance(service: *mut c_void) -> Result<u16, i32> {
    // Prepare read packet
    let mut data = [0u8; 16];
    data[0] = 0x82;
    data[1] = 0x01;
    data[2] = 0x10; // LUMINANCE
    data[3] = 0x6e ^ data[0] ^ data[1] ^ data[2];

    // Send read request
    sleep(DDC_WAIT);
    let ret = unsafe { IOAVServiceWriteI2C(service, 0x37, INPUT_ADDR as u32, data.as_ptr(), 4) };
    if ret != 0 {
        return Err(ret);
    }

    // Read response
    let mut response = [0u8; 12];
    sleep(DDC_WAIT);
    let ret = unsafe {
        IOAVServiceReadI2C(
            service,
            0x37,
            INPUT_ADDR as u32,
            response.as_mut_ptr(),
            12,
        )
    };
    if ret != 0 {
        return Err(ret);
    }

    Ok(((response[8] as u16) << 8) | response[9] as u16)
}

fn set_luminance(service: *mut c_void, value: u16) -> Result<(), i32> {
    let mut data = [0u8; 8];
    data[0] = 0x84;
    data[1] = 0x03;
    data[2] = 0x10; // LUMINANCE
    data[3] = (value >> 8) as u8;
    data[4] = (value & 0xff) as u8;
    data[5] = 0x6e ^ INPUT_ADDR ^ data[0] ^ data[1] ^ data[2] ^ data[3] ^ data[4];

    for _ in 0..2 {
        sleep(DDC_WAIT);
        let ret = unsafe { IOAVServiceWriteI2C(service, 0x37, INPUT_ADDR as u32, data.as_ptr(), 6) };
        if ret != 0 {
            return Err(ret);
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "help" {
        println!("Controls luminance (brightness) of an external display over DDC.");
        println!();
        println!("Usage:");
        println!(" set-luminance         - Returns current luminance");
        println!(" set-luminance 75      - Sets luminance to 75 (0-100)");
        println!(" set-luminance help    - Shows this help message");
        return;
    }

    let service = unsafe { IOAVServiceCreate(null()) };
    if service.is_null() {
        eprintln!("Could not find a suitable external display.");
        exit(1);
    }

    match args.len() {
        1 => match get_luminance(service) {
            Ok(v) => println!("{v}"),
            Err(e) => {
                eprintln!("DDC read failed: {e}");
                exit(1);
            }
        },
        2 => {
            let value: u16 = args[1].parse().unwrap_or(0).min(100);
            if let Err(e) = set_luminance(service, value) {
                eprintln!("DDC write failed: {e}");
                exit(1);
            }
        }
        _ => {
            eprintln!("Usage: set-luminance [value]");
            exit(1);
        }
    }
}

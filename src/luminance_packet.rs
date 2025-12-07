use std::fmt::Debug;

pub struct LuminancePacket {
    pub data: [u8; 16],
}

impl Debug for LuminancePacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X?}", &self.data[..16])
    }
}

impl LuminancePacket {
    pub fn new_write_packet(value: u8) -> Self {
        let mut packet = Self { data: [0; 16] };
        packet.data[0] = 0x84; // Write command
        packet.data[1] = 0x03; // Length
        packet.data[2] = 0x10; // DDC command for luminance
        packet.data[3] = 0x00; // High byte (always 0 for 8-bit values)
        packet.data[4] = value.clamp(0, 100); // Low byte (actual value)
        packet.data[5] = 0x6E
            ^ 0x51
            ^ packet.data[0]
            ^ packet.data[1]
            ^ packet.data[2]
            ^ packet.data[3]
            ^ packet.data[4]; // Checksum

        packet
    }

    pub fn new_read_packet() -> Self {
        let mut packet = Self { data: [0; 16] };
        packet.data[0] = 0x82; // Read command
        packet.data[1] = 0x01; // Length
        packet.data[2] = 0x10; // DDC command for luminance
        packet.data[3] = (0x6E ^ 0x50) ^ packet.data[0] ^ packet.data[1] ^ packet.data[2]^ packet.data[3]^ packet.data[4]; // Checksum
        packet
    }
}

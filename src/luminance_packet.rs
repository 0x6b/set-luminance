const DEFAULT_INPUT_ADDRESS: u8 = 0x6e;

pub struct LuminancePacket {
    pub data: [u8; 128],
    pub input_addr: u8,
}

impl From<u8> for LuminancePacket {
    fn from(value: u8) -> Self {
        // Clamp the value to 100
        let value = if value > 100 { 100 } else { value };

        // Create the packet
        let mut packet = Self { data: [0; 128], input_addr: DEFAULT_INPUT_ADDRESS };
        packet.data[0] = 0x84; // Write command
        packet.data[1] = 0x03; // Length
        packet.data[2] = 0x10; // DDC command for luminance
        packet.data[3] = 0x00; // High byte (always 0 for 8-bit values)
        packet.data[4] = value; // Low byte (actual value)
        // Calculate the checksum
        packet.data[5] = 0x6E
            ^ 0x51
            ^ packet.data[0]
            ^ packet.data[1]
            ^ packet.data[2]
            ^ packet.data[3]
            ^ packet.data[4];

        packet
    }
}

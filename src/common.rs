
pub const MNGP_PRIME: u8 = 0x00;
pub const MNGP_PRIME_GETQRY: u8 = 0x00;
pub const MNGP_PRIME_GETRSP: u8 = 0x01;
pub const MNGP_PRIME_SET: u8 = 0x02;
pub const MNGP_PRIME_RESET: u8 = 0x03;
pub const MNGP_PRIME_REBOOT: u8 = 0x04;
pub const MNGP_PRIME_FU: u8 = 0x05;
pub const MNGP_PRIME_EN_PIBQRY: u8 = 0x06;
pub const MNGP_PRIME_EN_PIBRSP: u8 = 0x07;
// Atmel serialized protocols
pub const PROTOCOL_SNIF_PRIME: u8 = 0x13;
pub const PROTOCOL_MAC_PRIME: u8 = 0x17;
pub const PROTOCOL_MLME_PRIME: u8 = 0x18;
pub const PROTOCOL_PLME_PRIME: u8 = 0x19;
pub const PROTOCOL_432_PRIME: u8 = 0x1A;
pub const PROTOCOL_BASEMNG_PRIME: u8 = 0x1D;
pub const PROTOCOL_PRIMEoUDP: u8 = 0x1F;
pub const PROTOCOL_PHY_ATPL2x0: u8 = 0x22;

pub const PROTOCOL_ATPL230: u8 = PROTOCOL_PHY_ATPL2x0;
pub const PROTOCOL_ATPL250: u8 = PROTOCOL_PHY_ATPL2x0;
pub const PROTOCOL_SNIF_G3: u8 = 0x23;
pub const PROTOCOL_MAC_G3: u8 = 0x24;
pub const PROTOCOL_ADP_G3: u8 = 0x25;
pub const PROTOCOL_COORD_G3: u8 = 0x26;
pub const PROTOCOL_PRIME_API: u8 = 0x30;
pub const PROTOCOL_USER_DEFINED: u8 = 0x3E;
pub const PROTOCOL_USER_DEFINED_2: u8 = 0xFE;
pub const PROTOCOL_INVALID: u8 = 0xFF;

pub const USI_LOG_LEVEL_ERR: u8 = 3;
pub const USI_LOG_LEVEL_INFO: u8 = 2;
pub const USI_LOG_LEVEL_DEBUG: u8 = 1;

pub const USI_LOG_LEVEL: u8 = USI_LOG_LEVEL_ERR;

pub const MSGMARK: u8 = 0x7e;
pub const ESCMARK: u8 = 0x7d;

pub const HEADER_LEN: u8 = 2;
pub const CRC8_LEN: u8 = 1;
pub const CRC16_LEN: u8 = 2;
pub const CRC32_LEN: u8 = 4;

pub const TYPE_PROTOCOL_OFFSET: u8 = 1;
pub const TYPE_PROTOCOL_MSK: u8 = 0x3F;

pub const LEN_PROTOCOL_HI_OFFSET: u8 = 0;
pub const LEN_PROTOCOL_HI_MSK: u8 = 0xFF;
pub const LEN_PROTOCOL_HI_SHIFT: u8 = 2;

pub const LEN_PROTOCOL_LO_OFFSET: u8 = 1;
pub const LEN_PROTOCOL_LO_MSK: u8 = 0xC0;
pub const LEN_PROTOCOL_LO_SHIFT: u8 = 6;

pub const XLEN_PROTOCOL_OFFSET: u8 = 2;
pub const XLEN_PROTOCOL_MSK: u8 = 0x80;
pub const XLEN_PROTOCOL_SHIFT_L: u8 = 3;
pub const XLEN_PROTOCOL_SHIFT_R: u8 = 10;

pub const PAYLOAD_OFFSET: u8 = 2;

pub const CMD_PROTOCOL_OFFSET: u8 = 2;
pub const CMD_PROTOCOL_MSK: u8 = 0x7F;

pub const PROTOCOL_DELIMITER: u8 = 0x7e;
pub const PROTOCOL_ESC: u8 = 0x7d;
pub const PROTOCOL_MIN_LEN: u8 = 4;

pub fn get_protocol_len(A: u16, B: u16) -> u16 {
    return (A << LEN_PROTOCOL_HI_SHIFT) + (B >> LEN_PROTOCOL_LO_SHIFT);
}
pub fn get_protocol_xlen(A: u8, B: u8, C: u8) -> u16 {
    return ((A as u16) << LEN_PROTOCOL_HI_SHIFT)
        + ((B as u16) >> LEN_PROTOCOL_LO_SHIFT)
        + (((C & XLEN_PROTOCOL_MSK) as u16)
        << XLEN_PROTOCOL_SHIFT_L);
}

pub fn LEN_HI_PROTOCOL(len: u16) -> u8 {
    ((len >> LEN_PROTOCOL_HI_SHIFT) & LEN_PROTOCOL_HI_MSK as u16) as u8
}
pub fn LEN_LO_PROTOCOL(len: u16) -> u8 {
    ((len << LEN_PROTOCOL_LO_SHIFT) & LEN_PROTOCOL_LO_MSK as u16) as u8
}
pub fn LEN_EX_PROTOCOL(len: u16) -> u8 {
    return ((len & 0x0c00) >> 4) as u8;
}
pub fn CMD_PROTOCOL(cmd: u8) -> u8 {
    cmd & CMD_PROTOCOL_MSK
}
pub fn TYPE_PROTOCOL(A: u8) -> u8 {
    (A) & TYPE_PROTOCOL_MSK
}


pub fn to_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs.join(" ")
}
pub fn array_to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs.join(" ")
}

#[derive(Clone)]
pub struct Parameter {
    pub protocol: u8,
    pub id: u32,
    pub idx: u16,
    pub value: Vec<u8>,
}
impl Parameter {
    pub fn new(protocol: u8, id: u32, idx: u16, value: Vec<u8>) -> Self {
        Parameter {
            protocol,
            id,
            idx,
            value,
        }
    }
}
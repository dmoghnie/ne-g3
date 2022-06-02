use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use crate::adp;
use crate::adp::TExtendedAddress;

// /// The LBD requests joining a PAN and provides the necessary authentication material.
// #define LBP_JOINING 0x01

// /// Authentication succeeded with delivery of device specific information (DSI) to the LBD
// #define LBP_ACCEPTED 0x09

// /// Authentication in progress. PAN specific information (PSI) may be delivered to the LBD
// #define LBP_CHALLENGE 0x0A

// /// Authentication failed
// #define LBP_DECLINE 0x0B

// /// KICK frame is used by any device to inform the coordinator that it left the PAN.
// #define LBP_KICK_FROM_LBD 0x04

// /// KICK frame is used by a PAN coordinator to force a device to lose its MAC address
// #define LBP_KICK_TO_LBD 0x0C



#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LbpMessageType {
    LBP_JOINING = 0x01,
    LBP_ACCEPTED = 0x09,
    LBP_CHALLENGE = 0x0A,
    LBP_DECLINE = 0x0B,
    LBP_KICK_FROM_LBD = 0x04,
    LBP_KICK_TO_LBD = 0x0C,
}

#[derive(Debug)]
pub enum LbpMessage {
    Joining(JoiningMessage),
    Accepted(AcceptedMessage),
    Challenge(ChallengeMessage),
    Decline(DeclineMessage),
    KickFromLbd(KickFromLbdMessage),
    KickToLbd(KickToLbdMessage),
}

#[derive(Debug)]
pub struct JoiningMessage {
    pub ext_addr: TExtendedAddress,
    pub bootstrapping_data: Vec<u8>,
}

// pdata[u16PDataLen++] = CONF_PARAM_SHORT_ADDR;
// 			pdata[u16PDataLen++] = 2;
// 			pdata[u16PDataLen++] = (unsigned char)((u16ShortAddr & 0xFF00) >> 8);
// 			pdata[u16PDataLen++] = (unsigned char)(u16ShortAddr & 0x00FF);

#[derive(Debug)]
pub struct AcceptedMessage {
    pub ext_addr: TExtendedAddress,

    pub bootstrapping_data: Vec<u8>,
}
// impl AcceptedMessage {
//     pub fn new(ext_addr: TExtendedAddress, short_addr: u16) -> Self {
//         // let mut v = vec![];

//         let mut v = vec![CONF_PARAM_SHORT_ADDR];
//         v.push(0x2);
//         for ch in short_addr.to_be_bytes(){
//             v.push(ch);
//         }
//         // v.push(((short_addr & 0xFF00) >> 8) as u8);
//         // v.push((short_addr & 0x00FF) as u8);
        
//         AcceptedMessage { ext_addr: ext_addr, bootstrapping_data: v }
//     }
// }
/*
impl Into<Vec<u8>> for AcceptedMessage {
    fn into(mut self) -> Vec<u8> {
        let cmd: u8 = LbpMessageType::LBP_ACCEPTED.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        // v.append(&mut self.bootstrapping_data);
        v.push(CONF_PARAM_SHORT_ADDR);
        v.push(((self.short_addr & 0xFF00) >> 8) as u8);
        v.push((self.short_addr & 0x00FF) as u8);
        return v;
    }
} */

impl Into<Vec<u8>> for AcceptedMessage {
    fn into(mut self) -> Vec<u8> {
        // let cmd: u8 = LbpMessageType::LBP_ACCEPTED.into();
        // let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        // v.append(&mut self.ext_addr.0.to_vec());
        // v.append(&mut self.bootstrapping_data);
        // return v;
        let cmd: u8 = LbpMessageType::LBP_ACCEPTED.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        let mut a = self.ext_addr.0.to_vec().clone();        
        // a.reverse();
        v.append(&mut a);
        v.append(&mut self.bootstrapping_data);
        return v;
    }
}

#[derive(Debug)]
pub struct ChallengeMessage {
    pub ext_addr: TExtendedAddress,
    pub bootstrapping_data: Vec<u8>,
}
impl Into<Vec<u8>> for ChallengeMessage {
    fn into(mut self) -> Vec<u8> {
        let cmd: u8 = LbpMessageType::LBP_CHALLENGE.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        v.append(&mut self.bootstrapping_data);
        return v;
    }
}
#[derive(Debug)]
pub struct DeclineMessage {
    pub ext_addr: TExtendedAddress,
}
impl DeclineMessage {
    pub fn new(ext_addr: TExtendedAddress)->Self{
        DeclineMessage { ext_addr }
    }
}
impl Into<Vec<u8>> for DeclineMessage {
    fn into(self) -> Vec<u8> {
        let cmd: u8 = LbpMessageType::LBP_DECLINE.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        let mut a = self.ext_addr.0.to_vec().clone();
        // a.reverse();
        v.append(&mut a);
        return v;
    }
}

#[derive(Debug)]
pub struct KickFromLbdMessage {
    ext_addr: TExtendedAddress,
}
impl Into<Vec<u8>> for KickFromLbdMessage {
    fn into(self) -> Vec<u8> {
        let cmd: u8 = LbpMessageType::LBP_KICK_FROM_LBD.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        return v;
    }
}

#[derive(Debug)]
pub struct KickToLbdMessage {
    ext_addr: TExtendedAddress,
}
impl Into<Vec<u8>> for KickToLbdMessage {
    fn into(self) -> Vec<u8> {
        let cmd: u8 = LbpMessageType::LBP_KICK_TO_LBD.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        return v;
    }
}

/*
LBP message format

T : 1 bit: Identifies the type of message 0: Message from LBD 1: Message to LBD
Code : 3 bits: Identifies the message code
Transaction-id: 12 bits: Reserved by ITU-T, set to 0 by the sender and ignored by the receiver.
A_LBD: 8 bytes: Indicates the EUI-64 address of the bootstrapping device (LBD). 
*/

pub const LBP_MESSAGE_MIN_LEN:usize = adp::ADP_ADDRESS_64BITS + 2usize; // is the sum of both T and Code

pub fn adp_message_to_lbp_message(msg: &adp::AdpG3LbpEvent) -> Option<LbpMessage> {
    // u8MessageType = ((pNsdu[0] & 0xF0) >> 4);
    if msg.nsdu.len() < LBP_MESSAGE_MIN_LEN {
        return None;
    }
    if let Ok(lbp_msg_type) = LbpMessageType::try_from_primitive (((msg.nsdu[0] & 0xF0) >> 4)) {
        
        if let Ok(ext_addr) = msg.nsdu[2..(LBP_MESSAGE_MIN_LEN)].try_into() {   
            
        match lbp_msg_type {
            LbpMessageType::LBP_JOINING => {                
                return Some(LbpMessage::Joining(JoiningMessage {ext_addr: ext_addr, bootstrapping_data: msg.nsdu[(LBP_MESSAGE_MIN_LEN)..].to_vec()}));
            }
            LbpMessageType::LBP_ACCEPTED => {
                return Some(LbpMessage::Accepted(AcceptedMessage {ext_addr: ext_addr, bootstrapping_data: msg.nsdu[(LBP_MESSAGE_MIN_LEN)..].to_vec()}));
            },
            LbpMessageType::LBP_CHALLENGE => {
                return Some(LbpMessage::Challenge(ChallengeMessage {ext_addr: ext_addr, bootstrapping_data: msg.nsdu[(LBP_MESSAGE_MIN_LEN)..].to_vec()}));
            },
            LbpMessageType::LBP_DECLINE => {
                return Some(LbpMessage::Decline(DeclineMessage {ext_addr: ext_addr}));
            },
            LbpMessageType::LBP_KICK_FROM_LBD => {
                return Some(LbpMessage::KickFromLbd(KickFromLbdMessage {ext_addr: ext_addr}));
            },
            LbpMessageType::LBP_KICK_TO_LBD => {
                return Some(LbpMessage::KickToLbd(KickToLbdMessage {ext_addr: ext_addr}));
            },
        }
    }
    }
    log::warn!("adp_msg_to_message_lbp for lpb_msg failed ");
    None
}

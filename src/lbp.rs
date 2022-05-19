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
    LBP_KICK_TO_LBD = 0x0C
}

pub enum LbpMessage {
    Joining (JoiningMessage),
    Accepted(AcceptedMessage),
    Challenge(ChallengeMessage),
    Decline(DeclineMessage),
    KickFromLbd(KickFromLbdMessage),
    KickToLbd(KickToLbdMessage)
}
pub struct JoiningMessage {

}
pub struct AcceptedMessage {
    ext_addr: TExtendedAddress,
    bootstrapping_data: Vec<u8>
}
impl Into<Vec<u8>> for AcceptedMessage {
    fn into(mut self) -> Vec<u8> {
        let cmd:u8 = LbpMessageType::LBP_ACCEPTED.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        v.append(&mut self.bootstrapping_data);
        return v;
    }
}


pub struct ChallengeMessage {
    ext_addr: TExtendedAddress,
    bootstrapping_data: Vec<u8>
}
impl Into<Vec<u8>> for ChallengeMessage {
    fn into(mut self) -> Vec<u8> {
        let cmd:u8 = LbpMessageType::LBP_CHALLENGE.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        v.append(&mut self.bootstrapping_data);
        return v;
    }
}
pub struct DeclineMessage {
    ext_addr: TExtendedAddress
}
impl Into<Vec<u8>> for DeclineMessage {
    fn into(self) -> Vec<u8> {
        let cmd:u8 = LbpMessageType::LBP_DECLINE.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        return v;
    }
}

pub struct KickFromLbdMessage {
    ext_addr: TExtendedAddress
}
impl Into<Vec<u8>> for KickFromLbdMessage {
    fn into(self) -> Vec<u8> {
        let cmd:u8 = LbpMessageType::LBP_KICK_FROM_LBD.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        return v;
    }
}


pub struct KickToLbdMessage {
    ext_addr: TExtendedAddress,

}
impl Into<Vec<u8>> for KickToLbdMessage {
    fn into(self) -> Vec<u8> {
        let cmd:u8 = LbpMessageType::LBP_KICK_TO_LBD.into();
        let mut v = vec![(cmd << 4), 0x0 /*transaction id is reserved */];
        v.append(&mut self.ext_addr.0.to_vec());
        return v;
    }
}

pub fn adp_message_to_message (msg: &adp::Message) -> Option<LbpMessage> {

    None
}
use std::borrow::Borrow;
use std::borrow::BorrowMut;

use crate::adp::TAddress;
use crate::adp::TAdpBand;
use crate::common;
use crate::adp;
use crate::usi;
use crate::usi::OutMessage;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

#[derive(Debug)]
pub struct AdpInitializeRequest {    
    band: u8
}
impl AdpInitializeRequest {
    pub fn new(band:u8) -> AdpInitializeRequest {
        AdpInitializeRequest{           
            band
        }
    }
    pub fn from_band(band: &adp::TAdpBand ) -> AdpInitializeRequest {
        AdpInitializeRequest { band: TAdpBand::into(band.clone()) }
    }
}

impl Into<usi::OutMessage> for AdpInitializeRequest {    
    fn into(self) -> usi::OutMessage{
        let v = [adp::G3_SERIAL_MSG_ADP_INITIALIZE, self.band];        
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }
}

#[derive(Debug)]
pub struct AdpDiscoveryRequest {
    //The number of seconds the scan shall last.
    duration:u8 //maybe we should have durations in a more rusty way, then we convert to u8
}
impl AdpDiscoveryRequest {
    pub fn new (duration_secs: u8) -> Self {
        AdpDiscoveryRequest { duration: duration_secs}
    }
}
impl Into<usi::OutMessage> for AdpDiscoveryRequest {
    // fn to_command (&self) -> usi::cmd::Command {
    //     let v = vec![common::G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST, self.duration];
    //     usi::cmd::Command::new(usi::common::PROTOCOL_ADP_G3, &v)
    // }
    fn into(self) -> usi::OutMessage {
        let v = [adp::G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST, self.duration];
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }

}

// void AdpLbpRequest(const struct TAdpAddress *pDstAddr, uint16_t u16NsduLength,
//     uint8_t *pNsdu, uint8_t u8NsduHandle, uint8_t u8MaxHops,
//     bool bDiscoveryRoute, uint8_t u8QualityOfService, bool bSecurityEnable){
#[derive(Debug)]
    pub struct AdpLbpRequest {    
    dst_addr: adp::TAddress,    
    data: Vec<u8>,
    handle: u8,
    max_hops: u8,
    discover_route: bool,
    quality_of_service: u8,
    security_enable: bool
}

impl AdpLbpRequest {
    pub fn new(dst_addr: TAddress, data: Vec<u8>, handle: u8, max_hops: u8, discover_route: bool, quality_of_service:u8, security_enable: bool) -> AdpLbpRequest {
        AdpLbpRequest { dst_addr, data, handle, max_hops, discover_route, quality_of_service, security_enable }
    }
}

impl Into<usi::OutMessage> for AdpLbpRequest {

    fn into(mut self) -> usi::OutMessage {

        let mut address_v:Vec<u8> = self.dst_addr.into();        
        let mut  v = vec![adp::G3_SERIAL_MSG_ADP_LBP_REQUEST, self.handle,  self.max_hops, self.discover_route as u8,
            self.quality_of_service, self.security_enable as u8];
            
        // let address_len = address_v.len() as u16;
        // v.push((address_len >> 8) as u8);
        v.push((address_v.len() as u8));
        // v.append(&mut address_len.to_be_bytes().to_vec());
        let data_len = self.data.len() as u16;
        // v.push((data_len >> 8) as u8);
        // v.push((data_len as u8));
        
        v.append(&mut data_len.to_be_bytes().to_vec());
        v.append(&mut address_v);
        v.append(&mut self.data);
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }

}

#[derive(Debug)]
pub struct AdpNetworkStartRequest {    
    pan_id:u16 //maybe we should have durations in a more rusty way, then we convert to u8
}
impl AdpNetworkStartRequest {
    pub fn new(pan_id: u16) -> Self {
        AdpNetworkStartRequest { pan_id}
    }
}
impl Into<usi::OutMessage> for AdpNetworkStartRequest {
    fn into(self) -> usi::OutMessage {
        let pan_id_v = self.pan_id.to_be_bytes();
        let v = [adp::G3_SERIAL_MSG_ADP_NETWORK_START_REQUEST, pan_id_v[0], pan_id_v[1]];
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }

}

pub struct AdpGetRequest {
    attribute_id: adp::EAdpPibAttribute, 
    attribute_idx: u16
}
impl  AdpGetRequest  {
    pub fn new(attribute_id: adp::EAdpPibAttribute, attribute_idx: u16) -> AdpGetRequest {
        AdpGetRequest{
            attribute_id, attribute_idx
        }
    }
}

impl Into<usi::OutMessage> for AdpGetRequest{
    fn into(self) -> usi::OutMessage {
        let attribute: u32 = self.attribute_id.into();
        let attribute_v = attribute.to_be_bytes();
        // let v = [adp::G3_SERIAL_MSG_ADP_GET_REQUEST, 
        //     ((attribute >> 24) & 0xFF) as u8,
        //     ((attribute >> 16) & 0xFF) as u8,
        //     ((attribute >> 8) & 0xFF) as u8,
        //     ((attribute) & 0xFF) as u8,
        //     (self.attribute_idx >> 8) as u8,
        //     (self.attribute_idx & 0xFF) as u8];
        let mut v = vec![adp::G3_SERIAL_MSG_ADP_GET_REQUEST];
        for ch in attribute_v {
            v.push(ch);
        }
        for ch in self.attribute_idx.to_be_bytes() {
            v.push (ch);
        }
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }


}

pub struct AdpSetRequest<'a> {
    attribute_id: adp::EAdpPibAttribute, 
    attribute_idx: u16,
    attribute_value: &'a Vec<u8>
}
impl <'a> AdpSetRequest <'a> {
    pub fn new(attribute_id: adp::EAdpPibAttribute, attribute_idx: u16, attribute_value:&'a Vec<u8>) -> AdpSetRequest<'a> {
        AdpSetRequest{
            attribute_id, attribute_idx, attribute_value
        }
    }
}

impl Into<usi::OutMessage> for AdpSetRequest<'_>{
    
    fn into(self) -> usi::OutMessage {
        let attribute: u32 = self.attribute_id.into();
        let mut v = vec!(adp::G3_SERIAL_MSG_ADP_SET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8, (self.attribute_value.len() as u8));
        // for ch in self.attribute_value {
        //     v.push(ch);
        // }
        v.extend_from_slice(self.attribute_value);
        // let mut v = vec![adp::G3_SERIAL_MSG_ADP_SET_REQUEST];
        // for ch in attribute.to_le_bytes() {
        //     v.push(ch);
        // }
        // for ch in self.attribute_idx.to_le_bytes() {
        //     v.push(ch);
        // }
        // for ch in self.attribute_value {
        //     v.push(*ch);
        // }

        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }
}

pub struct AdpMacGetRequest {
    attribute_id: adp::EMacWrpPibAttribute, 
    attribute_idx: u16
}
impl  AdpMacGetRequest  {
    pub fn new(attribute_id: adp::EMacWrpPibAttribute, attribute_idx: u16) -> AdpMacGetRequest {
        AdpMacGetRequest{
            attribute_id, attribute_idx
        }
    }
}

impl Into<usi::OutMessage> for AdpMacGetRequest{
    
    fn into(self) -> usi::OutMessage {
        let attribute: u32 = self.attribute_id.into();
        let v = [adp::G3_SERIAL_MSG_ADP_MAC_GET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8];
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }
}

pub struct AdpMacSetRequest<'a> {
    attribute_id: adp::EMacWrpPibAttribute, 
    attribute_idx: u16,
    attribute_value: &'a Vec<u8>
}
impl  <'a>AdpMacSetRequest<'a>  {
    pub fn new(attribute_id: adp::EMacWrpPibAttribute, attribute_idx: u16, attribute_value:&'a Vec<u8>) -> AdpMacSetRequest<'a> {
        AdpMacSetRequest{
            attribute_id, attribute_idx, attribute_value
        }
    }
}

impl Into<usi::OutMessage> for AdpMacSetRequest<'_>{
    
    fn into(self) -> usi::OutMessage {
        let attribute: u32 = self.attribute_id.into();
        let mut v = vec!(adp::G3_SERIAL_MSG_ADP_MAC_SET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8, self.attribute_value.len() as u8);
        // for ch in self.attribute_value {
        //     v.push(ch);
        // }
        v.extend_from_slice(self.attribute_value);
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }
}

pub struct AdpDataRequest {    
    handle: u8,
    data: Vec<u8>,
    discover_route: bool,
    quality_of_service: u8
}

impl AdpDataRequest {
    pub fn new(handle: u8, data: &Vec<u8>, discover_route: bool, quality_of_service: u8)->AdpDataRequest {
        AdpDataRequest { handle, data: data.to_vec(), discover_route, quality_of_service }
    }
}

impl Into<usi::OutMessage> for AdpDataRequest {
    
    fn into(self) -> usi::OutMessage {        
        let mut v = vec![adp::G3_SERIAL_MSG_ADP_DATA_REQUEST, self.handle, 
            self.discover_route as u8, self.quality_of_service,
            (self.data.len() >> 8) as u8,
            (self.data.len() & 0xFF) as u8];
       for ch in self.data {
           v.push(ch);
       }
        OutMessage::new(common::PROTOCOL_ADP_G3, &v)
    }
}

pub struct AdpJoinNetworkRequest {
    pub pan_id: u16,
    pub lba_address: u16
}

impl Into<usi::OutMessage> for AdpJoinNetworkRequest {

    fn into(self) -> usi::OutMessage {
        // let v = [adp::G3_SERIAL_MSG_ADP_NETWORK_JOIN_REQUEST, (self.pan_id >> 8) as u8, (self.pan_id & 0xFF) as u8,
        //     (self.lba_address >> 8) as u8, (self.lba_address & 0xFF) as u8];        

        let mut v = vec![adp::G3_SERIAL_MSG_ADP_NETWORK_JOIN_REQUEST];
        for ch in self.pan_id.to_be_bytes() {
            v.push(ch);
        }
        for ch in self.lba_address.to_be_bytes() {
            v.push(ch);
        }
        OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec())
    }
}
use crate::adp::TAddress;
use crate::common;
use crate::adp;
use crate::usi;
use crate::usi::OutMessage;

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
    pub fn from_band(band:adp::TAdpBand ) -> AdpInitializeRequest {
        AdpInitializeRequest { band: band.into() }
    }
}

impl TryInto<usi::OutMessage> for AdpInitializeRequest {
    type Error = ();
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let v = [adp::G3_SERIAL_MSG_ADP_INITIALIZE, self.band];        
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }
}

#[derive(Debug)]
pub struct AdpDiscoveryRequest {
    //The number of seconds the scan shall last.
    duration:u8 //maybe we should have durations in a more rusty way, then we convert to u8
}
impl AdpDiscoveryRequest {
    pub fn new (duration: u8) -> Self {
        AdpDiscoveryRequest { duration}
    }
}
impl TryInto<usi::OutMessage> for AdpDiscoveryRequest {
    type Error=();
    // fn to_command (&self) -> usi::cmd::Command {
    //     let v = vec![common::G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST, self.duration];
    //     usi::cmd::Command::new(usi::common::PROTOCOL_ADP_G3, &v)
    // }
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let v = [adp::G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST, self.duration];
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }

}

// void AdpLbpRequest(const struct TAdpAddress *pDstAddr, uint16_t u16NsduLength,
//     uint8_t *pNsdu, uint8_t u8NsduHandle, uint8_t u8MaxHops,
//     bool bDiscoveryRoute, uint8_t u8QualityOfService, bool bSecurityEnable){
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

impl TryInto<usi::OutMessage> for AdpLbpRequest {
    type Error=();
   
    fn try_into(mut self) -> Result<usi::OutMessage, Self::Error> {

        let mut address_v:Vec<u8> = self.dst_addr.into();
        let mut  v = vec![adp::G3_SERIAL_MSG_ADP_LBP_REQUEST, self.handle,  self.max_hops, self.discover_route as u8,
            self.quality_of_service, self.security_enable as u8];
        v.append(&mut address_v.len().to_be_bytes().to_vec());
        v.append(&mut self.data.len().to_be_bytes().to_vec());
        v.append(&mut address_v);
        v.append(&mut self.data);
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
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
impl TryInto<usi::OutMessage> for AdpNetworkStartRequest {
    type Error=();
    
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let v = [adp::G3_SERIAL_MSG_ADP_NETWORK_START_REQUEST, (self.pan_id >> 8) as u8, (self.pan_id & 0xFF) as u8];
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
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

impl TryInto<usi::OutMessage> for AdpGetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let v = [adp::G3_SERIAL_MSG_ADP_GET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8];
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }


}

pub struct AdpSetRequest {
    attribute_id: adp::EAdpPibAttribute, 
    attribute_idx: u16,
    attribute_value: Vec<u8>
}
impl  AdpSetRequest  {
    pub fn new(attribute_id: adp::EAdpPibAttribute, attribute_idx: u16, attribute_value:Vec<u8>) -> AdpSetRequest {
        AdpSetRequest{
            attribute_id, attribute_idx, attribute_value
        }
    }
}

impl TryInto<usi::OutMessage> for AdpSetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let mut v = vec!(adp::G3_SERIAL_MSG_ADP_SET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8, (self.attribute_value.len() as u8));
        for ch in self.attribute_value {
            v.push(ch);
        }
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
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

impl TryInto<usi::OutMessage> for AdpMacGetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let v = [adp::G3_SERIAL_MSG_ADP_MAC_GET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8];
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }
}

pub struct AdpMacSetRequest {
    attribute_id: adp::EMacWrpPibAttribute, 
    attribute_idx: u16,
    attribute_value: Vec<u8>
}
impl  AdpMacSetRequest  {
    pub fn new(attribute_id: adp::EMacWrpPibAttribute, attribute_idx: u16, attribute_value:Vec<u8>) -> AdpMacSetRequest {
        AdpMacSetRequest{
            attribute_id, attribute_idx, attribute_value
        }
    }
}

impl TryInto<usi::OutMessage> for AdpMacSetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let mut v = vec!(adp::G3_SERIAL_MSG_ADP_MAC_SET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8, self.attribute_value.len() as u8);
        for ch in self.attribute_value {
            v.push(ch);
        }
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
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

impl TryInto<usi::OutMessage> for AdpDataRequest {
    type Error = ();
    fn try_into(self) -> Result<usi::OutMessage, Self::Error> {
        let mut v = vec![adp::G3_SERIAL_MSG_ADP_DATA_REQUEST, self.handle, 
            self.discover_route as u8, self.quality_of_service,
            (self.data.len() >> 8) as u8,
            (self.data.len() & 0xFF) as u8];
       for ch in self.data {
           v.push(ch);
       }
        Ok(OutMessage::new(common::PROTOCOL_ADP_G3, &v))
    }
}
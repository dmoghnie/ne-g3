use crate::common;
use crate::message;
use crate::usi;
use crate::usi::UsiCommand;

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
    pub fn from_band(band:message::TAdpBand ) -> AdpInitializeRequest {
        AdpInitializeRequest { band: band.into() }
    }
}

impl TryInto<usi::UsiCommand> for AdpInitializeRequest {
    type Error = ();
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let v = [message::G3_SERIAL_MSG_ADP_INITIALIZE, self.band];        
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
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
impl TryInto<usi::UsiCommand> for AdpDiscoveryRequest {
    type Error=();
    // fn to_command (&self) -> usi::cmd::Command {
    //     let v = vec![common::G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST, self.duration];
    //     usi::cmd::Command::new(usi::common::PROTOCOL_ADP_G3, &v)
    // }
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let v = [message::G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST, self.duration];
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }

}

#[derive(Debug)]
pub struct AdpNetworkStartRequest {    
    pan_id:u16 //maybe we should have durations in a more rusty way, then we convert to u8
}
impl AdpNetworkStartRequest {
    pub fn new (pan_id: u16) -> Self {
        AdpNetworkStartRequest { pan_id}
    }
}
impl TryInto<usi::UsiCommand> for AdpNetworkStartRequest {
    type Error=();
    // fn to_command (&self) -> usi::cmd::Command {
    //     let v = vec![common::G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST, self.duration];
    //     usi::cmd::Command::new(usi::common::PROTOCOL_ADP_G3, &v)
    // }
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let v = [message::G3_SERIAL_MSG_ADP_NETWORK_START_REQUEST, (self.pan_id >> 8) as u8, (self.pan_id & 0xFF) as u8];
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }

}

pub struct AdpGetRequest {
    attribute_id: message::EAdpPibAttribute, 
    attribute_idx: u16
}
impl  AdpGetRequest  {
    pub fn new (attribute_id: message::EAdpPibAttribute, attribute_idx: u16) -> AdpGetRequest {
        AdpGetRequest{
            attribute_id, attribute_idx
        }
    }
}

impl TryInto<usi::UsiCommand> for AdpGetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let v = [message::G3_SERIAL_MSG_ADP_GET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8];
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }


}

pub struct AdpSetRequest {
    attribute_id: message::EAdpPibAttribute, 
    attribute_idx: u16,
    attribute_value: Vec<u8>
}
impl  AdpSetRequest  {
    pub fn new (attribute_id: message::EAdpPibAttribute, attribute_idx: u16, attribute_value:Vec<u8>) -> AdpSetRequest {
        AdpSetRequest{
            attribute_id, attribute_idx, attribute_value
        }
    }
}

impl TryInto<usi::UsiCommand> for AdpSetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let mut v = vec!(message::G3_SERIAL_MSG_ADP_SET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8, (self.attribute_value.len() as u8));
        for ch in self.attribute_value {
            v.push(ch);
        }
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }
}

pub struct AdpMacGetRequest {
    attribute_id: message::EMacWrpPibAttribute, 
    attribute_idx: u16
}
impl  AdpMacGetRequest  {
    pub fn new (attribute_id: message::EMacWrpPibAttribute, attribute_idx: u16) -> AdpMacGetRequest {
        AdpMacGetRequest{
            attribute_id, attribute_idx
        }
    }
}

impl TryInto<usi::UsiCommand> for AdpMacGetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let v = [message::G3_SERIAL_MSG_ADP_MAC_GET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8];
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }
}

pub struct AdpMacSetRequest {
    attribute_id: message::EMacWrpPibAttribute, 
    attribute_idx: u16,
    attribute_value: Vec<u8>
}
impl  AdpMacSetRequest  {
    pub fn new (attribute_id: message::EMacWrpPibAttribute, attribute_idx: u16, attribute_value:Vec<u8>) -> AdpMacSetRequest {
        AdpMacSetRequest{
            attribute_id, attribute_idx, attribute_value
        }
    }
}

impl TryInto<usi::UsiCommand> for AdpMacSetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let attribute: u32 = self.attribute_id.into();
        let mut v = vec!(message::G3_SERIAL_MSG_ADP_MAC_SET_REQUEST, 
            ((attribute >> 24) & 0xFF) as u8,
            ((attribute >> 16) & 0xFF) as u8,
            ((attribute >> 8) & 0xFF) as u8,
            ((attribute) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8, self.attribute_value.len() as u8);
        for ch in self.attribute_value {
            v.push(ch);
        }
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v.to_vec()))
    }
}

pub struct AdpDataRequest {    
    handle: u8,
    data: Vec<u8>,
    discover_route: bool,
    quality_of_service: u8
}

impl AdpDataRequest {
    pub fn new (handle: u8, data: &Vec<u8>, discover_route: bool, quality_of_service: u8)->AdpDataRequest {
        AdpDataRequest { handle, data: data.to_vec(), discover_route, quality_of_service }
    }
}

impl TryInto<usi::UsiCommand> for AdpDataRequest {
    type Error = ();
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let mut v = vec![message::G3_SERIAL_MSG_ADP_DATA_REQUEST, self.handle, 
            self.discover_route as u8, self.quality_of_service,
            (self.data.len() >> 8) as u8,
            (self.data.len() & 0xFF) as u8];
       for ch in self.data {
           v.push(ch);
       }
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v))
    }
}
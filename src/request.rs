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

pub struct AdpGetRequest {
    attribute_id:u32, 
    attribute_idx: u16
}
impl  AdpGetRequest  {
    pub fn new (attribute_id: u32, attribute_idx: u16) -> AdpGetRequest {
        AdpGetRequest{
            attribute_id, attribute_idx
        }
    }
}

impl TryInto<usi::UsiCommand> for AdpGetRequest{
    type Error = ();
    fn try_into(self) -> Result<usi::UsiCommand, Self::Error> {
        let v = [message::G3_SERIAL_MSG_ADP_GET_REQUEST, 
            ((self.attribute_id >> 24) & 0xFF) as u8,
            ((self.attribute_id >> 16) & 0xFF) as u8,
            ((self.attribute_id >> 8) & 0xFF) as u8,
            ((self.attribute_id) & 0xFF) as u8,
            (self.attribute_idx >> 8) as u8,
            (self.attribute_idx & 0xFF) as u8];
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
       for ch in &self.data {
           v.push(*ch);
       }
        Ok(UsiCommand::new(common::PROTOCOL_ADP_G3, &v))
    }
}
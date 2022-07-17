use crate::{usi, app_config, request::AdpMacGetRequest, adp::{EMacWrpPibAttribute, G3_SERIAL_MSG_MAC_GET_CONFIRM, self, TExtendedAddress}};

use super::{State, Stateful, Context, Response, Message};
use num_enum::TryFromPrimitive;

pub struct GetParams {
   
}

impl GetParams {
    pub fn new() -> Self {
        GetParams {
          
        }
    }
}

impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for GetParams {
    fn on_enter(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        context: &mut Context,
    ) -> Response<State> {
        log::info!("State : GetParams - onEnter");
        let request = AdpMacGetRequest::new (EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS, 0);
        cs.send(usi::Message::UsiOut(request.into()));
        Response::Handled
    }

    fn on_event(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        log::trace!("GetParams : {:?}", event);
        match event {
            Message::Adp(adp_msg) => {
                match adp_msg {
                    adp::Message::AdpG3GetMacResponse(mac_get_response) => {
                            if let Ok(attr) = EMacWrpPibAttribute::try_from(mac_get_response.attribute_id) {
                                match attr {
                                    EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS => {
                                        context.extended_addr = 
                                            TExtendedAddress::try_from(mac_get_response.attribute_val.as_slice()).map_or(None, |v| Some(v));
                                    },
                                    _ => {}
                                }
                            }
                        if context.is_coordinator {
                            return Response::Transition(State::StartNetwork);
                        }
                        else{
                            return Response::Transition(State::JoinNetwork);
                        }
                    }
                    _ => {

                    }
                }
            },
            Message::HeartBeat(_) => {},
            Message::Startup => {},
        }
        Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {}
}
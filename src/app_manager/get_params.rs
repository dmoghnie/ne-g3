use crate::{usi, app_config, request::{AdpMacGetRequest, AdpSetRequest}, adp::{EMacWrpPibAttribute, G3_SERIAL_MSG_MAC_GET_CONFIRM, self, TExtendedAddress, ipv6_prefix}};

use super::{State, Context, Message};
use nefsm::{Stateful, Response};
use num_enum::TryFromPrimitive;

pub struct GetParams {
   
}

impl GetParams {
    pub fn new() -> Self {
        GetParams {
          
        }
    }
}

impl Stateful<State, Context, Message<'_>> for GetParams {
    fn on_enter(
        &mut self,        
        context: &mut Context,
    ) -> Response<State> {
        log::info!("State : GetParams - onEnter");
        let request = AdpMacGetRequest::new (EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS, 0);
        context.usi_tx.send(usi::Message::UsiOut(request.into()));
        Response::Handled
    }

    fn on_event(
        &mut self,
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
                                       let mut v = mac_get_response.attribute_val.clone();
                                       v.reverse();
                                        context.extended_addr = 
                                            TExtendedAddress::try_from(v.as_slice()).map_or(None, |v| Some(v));
                                       if let Some(ipv6_addr) = 
                                        app_config::ula_ipv6_addr_from_pan_id_extended_addr(&context.settings.network.ula_net_prefix,
                                            context.settings.g3.pan_id, &context.extended_addr.unwrap()) {
                                        
                                        log::info!("Setting ipv6 network prefix");
                                        let v6prefix = ipv6_prefix::new(context.settings.network.ula_net_prefix_len, &ipv6_addr);
                                        // AdpSetRequestSync(ADP_IB_PREFIX_TABLE,0,sizeof(struct ipv6_prefix),(uint8_t*)&net_prefix, &pSetConfirm);
                                        unsafe {
                                            let v = v6prefix.to_raw_data().to_vec();
                                            let request = AdpSetRequest::new (adp::EAdpPibAttribute::ADP_IB_PREFIX_TABLE, 0, &v);
                                            context.usi_tx.send(usi::Message::UsiOut(request.into()));
                                        }
                                       }
                                    },
                                    _ => {}
                                }
                            }
                        if context.is_coordinator {
                            return Response::Transition(State::StartNetwork);
                        }
                        else{
                            return Response::Transition(State::DiscoverNetwork);
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
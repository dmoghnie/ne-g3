use std::collections::VecDeque;

use nefsm::{Stateful, Response};

use crate::{
    adp,
    app_config::{self, G3ParamType},
    request::{AdpMacSetRequest, AdpSetRequest},
    usi,
};

use super::{Context, Message, State};

pub struct SetParams {
    params: Option<VecDeque<app_config::G3Param>>,
}

impl SetParams {
    pub fn new() -> Self {
        SetParams { params: None }
    }
    fn init_params(&mut self, context: &Context) {
        if context.is_coordinator {
            let params = vec![
                (
                    G3ParamType::Mac,
                    adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID.into(),
                    0,
                    context.settings.g3.pan_id.to_be_bytes().to_vec(),
                ),
                (
                    G3ParamType::Mac,
                    adp::EMacWrpPibAttribute::MAC_WRP_PIB_KEY_TABLE.into(),
                    0,
                    context.settings.g3.gmk.to_vec(),
                ),
                //TODO rekey
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(),
                    0,
                    vec![0x05],
                ), //TODO, parameterize
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_ACTIVE_KEY_INDEX.into(),
                    0,
                    vec![0x00],
                ), //TODO parameterize
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME.into(),
                    0,
                    vec![0x10, 0x00],
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_MAX_HOPS.into(),
                    0,
                    vec![context.settings.g3.max_hops],
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY.into(),
                    0,
                    context.settings.g3.psk.to_vec(),
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(),
                    0,
                    context.settings.g3.context_information_table_0.clone(),
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(),
                    1,
                    context.settings.g3.context_information_table_1.clone(),
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(),
                    0,
                    vec![0xB4, 0x00],
                ),
                //TODO parameterize
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_COORD_SHORT_ADDRESS.into(),
                    0,
                    vec![0x00, 0x00],
                ),
                //TODO parameterize
                (
                    G3ParamType::Mac,
                    adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS.into(),
                    0,
                    vec![0x00, 0x00],
                ),
            ];
            self.params = Some(params.into());
        } else {
            let params = vec![
                (
                    G3ParamType::Mac,
                    adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID.into(),
                    0,
                    context.settings.g3.pan_id.to_be_bytes().to_vec()                    
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(),
                    0,
                    vec![0x05],
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME.into(),
                    0,
                    vec![0x10, 0x00],
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_MAX_HOPS.into(),
                    0,
                    vec![context.settings.g3.max_hops],
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY.into(),
                    0,
                    context.settings.g3.psk.to_vec()                    
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(),
                    0,
                    context.settings.g3.context_information_table_0.clone()
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(),
                    1,
                    context.settings.g3.context_information_table_1.clone()            
                ),
                (
                    G3ParamType::Adp,
                    adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(),
                    0,
                    vec![0xB4, 0x00],
                ),
                // (
                //     G3ParamType::Adp,
                //     adp::EAdpPibAttribute::ADP_IB_COORD_SHORT_ADDRESS.into(),
                //     0,
                //     vec![0x00, 0x01],
                // ),
            ];
            self.params = Some(params.into());
        }
    }
    fn set_param(&self, cs: &flume::Sender<usi::Message>, param: &app_config::G3Param) -> bool {
        let msg = if param.0 == G3ParamType::Mac {
            AdpMacSetRequest::new(param.1.try_into().unwrap(), param.2, &param.3).into()
        } else {
            AdpSetRequest::new(param.1.try_into().unwrap(), param.2, &param.3).into()
        };
        match cs.send(usi::Message::UsiOut(msg)) {
            Ok(_) => true,
            Err(e) => {
                log::warn!("Failed to set param : {:?} - {}", param, e);
                false
            }
        }
    }

    fn send_next_param(&mut self, cs: &flume::Sender<usi::Message>) -> bool {
        if let Some(params) = &mut self.params {
            if let Some(param) = params.pop_front() {
                return self.set_param(cs, &param);
            }
        }
        false
    }
}

impl Stateful<State, Context, Message<'_>> for SetParams {
    fn on_enter(
        &mut self,
        context: &mut Context,
    ) -> Response<State> {
        log::info!("State : SetParams - onEnter");
        self.init_params(context);
        self.send_next_param(&context.usi_tx);
        Response::Handled
    }

    fn on_event(
        &mut self,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        log::trace!("SetParams : {:?}", event);
        if self.send_next_param(&context.usi_tx) {
            Response::Handled
        } else {
            Response::Transition(State::GetParams)
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

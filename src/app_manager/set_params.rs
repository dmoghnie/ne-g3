use std::collections::VecDeque;

use crate::{
    app_config::{self, G3ParamType},
    request::{AdpMacSetRequest, AdpSetRequest},
    usi,
};

use super::{Context, Message, Response, State, Stateful};

pub struct SetParams {
    params: Option<VecDeque<app_config::G3Param>>,
}

impl SetParams {
    pub fn new() -> Self {
        SetParams {
           params: None,           
        }
    }
    fn set_param(
        &self,
        cs: &flume::Sender<usi::Message>,
        param: &app_config::G3Param,
    ) -> bool {
        let msg = 
            if param.0 == G3ParamType::Mac {AdpMacSetRequest::new(param.1.try_into().unwrap(), param.2, &param.3).into()} 
                else {AdpSetRequest::new(param.1.try_into().unwrap(), param.2, &param.3).into()};
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

impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for SetParams {
    fn on_enter(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        context: &mut Context,
    ) -> Response<State> {
        log::info!("State : SetParams - onEnter");
        if context.is_coordinator {
            self.params = Some(app_config::COORD_PARAMS.to_vec().into());
        } else {
            self.params =  Some(app_config::MODEM_PARAMS.to_vec().into());

        }
        self.send_next_param(cs);
        Response::Handled
    }

    fn on_event(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        log::trace!("SetParams : {:?}", event);
        if self.send_next_param(cs) {
            Response::Handled
        } else {
            Response::Transition(State::GetParams)
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

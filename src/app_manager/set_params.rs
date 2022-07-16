use crate::{app_config, usi, request::{AdpSetRequest, AdpMacSetRequest}};

use super::{Response, Stateful, State, Message};

pub struct SetParams {
    adp_params: Vec<app_config::AdpParam>,
    mac_params: Vec<app_config::MacParam>,
}

impl SetParams {
    pub fn new() -> Self {
        let (mac_params, adp_params) = app_config::COORD_PARAMS.clone();
        SetParams {
            adp_params,
            mac_params,
        }
    }
    fn set_adp_param(
        &self,
        cs: &flume::Sender<usi::Message>,
        param: &app_config::AdpParam,
    ) -> bool {
        let request = AdpSetRequest::new(param.0, param.1, &param.2);
        match cs.send(usi::Message::UsiOut(request.into())) {
            Ok(_) => {
                true
            },
            Err(e) => {
                log::warn!("Failed to set adp_param : {}", e);
                false
            },
        }
    }
    fn set_mac_param(&self, cs: &flume::Sender<usi::Message>, param: &app_config::MacParam) -> bool {
        let request = AdpMacSetRequest::new(param.0, param.1, &param.2);
        match cs.send(usi::Message::UsiOut(request.into())) {
            Ok(_) => {
                true
            },
            Err(e) => {
                log::warn!("Failed to set adp_param : {}", e);
                false
            },
        }
    }
    fn send_next_param(&mut self, cs: &flume::Sender<usi::Message>) -> bool {
        if self.mac_params.len() > 0 {
            if let Some(param) = self.mac_params.pop() {
                return self.set_mac_param(cs, &param);
            }
        } else if self.adp_params.len() > 0 {
            if let Some(param) = self.adp_params.pop() {
                return self.set_adp_param(cs, &param);
            }            
        }
        false
    }
}

impl Stateful<State, usi::Message, flume::Sender<usi::Message>> for SetParams {
    fn on_enter(&mut self, cs: &flume::Sender<usi::Message>) -> Response<State> {
        log::info!("State : SetParams - onEnter");
        
        self.send_next_param(cs);
        Response::Handled
    }

    fn on_event(&mut self, cs: &flume::Sender<usi::Message>, event: &Message) -> Response<State> {
        log::trace!("SetParams : {:?}", event);
        if self.send_next_param (cs) {
            Response::Handled
        }
        else{
            Response::Transition(State::Ready)
        }
    }

    fn on_exit(&mut self) {}
}
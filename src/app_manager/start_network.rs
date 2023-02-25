use nefsm::{Stateful, Response};

use crate::{usi, app_config, request, adp::{AdpG3NetworkStartResponse, self, EAdpStatus}};

use super::{State, Context, Message};

pub struct StartNetwork {
   
}

impl StartNetwork {
    pub fn new() -> Self {
        StartNetwork {
          
        }
    }
}

impl Stateful<State, Context, Message> for StartNetwork {
    fn on_enter(
        &mut self,
        context: &mut Context,
    ) -> Response<State> {
        log::info!("State : StartNetwork - onEnter : context {:?}", context);
        let cmd = request::AdpNetworkStartRequest::new(context.settings.g3.pan_id);
        if let Err(e) = context.usi_tx.send(usi::Message::UsiOut(cmd.into())) {
            log::warn!("Failed to send network start request {}", e);
        }
        Response::Handled
    }

    fn on_event(
        &mut self,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        log::trace!("StartNetwork : {:?}", event);
        match event {
            Message::Adp(adp) => {
                match adp {
                    adp::Message::AdpG3NetworkStartResponse(response) => {
                        if (response.status == EAdpStatus::G3_SUCCESS) {
                            return Response::Transition(State::Idle);
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }
        Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

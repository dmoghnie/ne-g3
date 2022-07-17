use crate::{usi, app_config, request};

use super::{State, Stateful, Context, Response, Message};

pub struct StartNetwork {
   
}

impl StartNetwork {
    pub fn new() -> Self {
        StartNetwork {
          
        }
    }
}

impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for StartNetwork {
    fn on_enter(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        context: &mut Context,
    ) -> Response<State> {
        log::info!("State : StartNetwork - onEnter : context {:?}", context);
        let cmd = request::AdpNetworkStartRequest::new(*app_config::PAN_ID);
        if let Err(e) = cs.send(usi::Message::UsiOut(cmd.into())) {
            log::warn!("Failed to send network start request {}", e);
        }
        Response::Handled
    }

    fn on_event(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        log::trace!("StartNetwork : {:?}", event);
        
        Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

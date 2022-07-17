use crate::{usi, app_config, request};

use super::{State, Stateful, Context, Response, Message};

pub struct JoinNetwork {
   
}

impl JoinNetwork {
    pub fn new() -> Self {
        JoinNetwork {
          
        }
    }
}

impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for JoinNetwork {
    fn on_enter(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        context: &mut Context,
    ) -> Response<State> {
        log::info!("State : JoinNetwork - onEnter : context {:?}", context);
        
        Response::Handled
    }

    fn on_event(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        log::trace!("JoinNetwork : {:?}", event);
        let cmd = request::AdpJoinNetworkRequest {
            pan_id: *app_config::PAN_ID,
            lba_address: 0,
        };
        if let Err(e) = cs.send(usi::Message::UsiOut(cmd.into())) {
            log::warn!("Failed to send network join request {}", e);
        }
        Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

use crate::{usi, app_config};

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
        log::info!("State : JoinNetwork - onEnter");
        
        Response::Handled
    }

    fn on_event(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        log::trace!("JoinNetwork : {:?}", event);
        
        Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

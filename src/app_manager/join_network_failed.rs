use nefsm::{Stateful, Response};

use crate::{usi, request::AdpSetRequest, adp::{EAdpPibAttribute, self}};

use super::{State, Context, Message};


pub struct JoinNetworkFailed {}
impl Stateful<State, Context, Message> for JoinNetworkFailed {
    fn on_enter(&mut self, context: &mut Context) -> Response<State> {
        
        Response::Handled
    }

    fn on_event(&mut self, event: &Message, context: &mut Context) -> Response<State> {
        log::info!("on event {:?}", event );
        match event {            
            Message::HeartBeat(time) => {
                Response::Transition(State::JoinNetwork)
            },
           _ => {                
                Response::Handled
            },
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}
use nefsm::{Stateful, Response};

use crate::{app_config, usi, request::{AdpSetRequest, AdpInitializeRequest, self}, app_manager::Idle, adp};

use super::{State, Message, Context};

pub struct Ready {

}

impl Ready {
    pub fn new() -> Self {
        
        Ready {
            
        }
    }
    
}

impl Stateful<State, Context, Message> for Ready {
    fn on_enter(&mut self, context: &mut Context) -> Response<State> {
        log::info!("State : Ready - onEnter");
        Response::Handled
    }

    fn on_event(&mut self, event: &Message, context: &mut Context) -> Response<State> {
        log::trace!("Ready : {:?}", event);
        match event {
            Message::Adp(adp) => {
                match adp {
                    adp::Message::AdpG3MsgStatusResponse(status_response) => {
                      Response::Handled
                    },                
                    _ => {
                        Response::Handled
                    }
                }
            },
            Message::HeartBeat(time) => {
                context.is_coordinator = true;
                Response::Transition(State::StackInitialize)
            }
            _ => {
                Response::Handled
            }
        }        
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

use crate::{app_config, usi, request::{AdpSetRequest, AdpInitializeRequest, self}, app_manager::Idle, adp::{self, TAdpBand}};

use nefsm::{Stateful, Response};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;


use super::{State, Message, Context};

pub struct StackInitialize {

}

impl StackInitialize {
    pub fn new() -> Self {
        
        StackInitialize {
            
        }
    }
    
}

impl Stateful<State, Context, Message> for StackInitialize {
    fn on_enter(&mut self, context: &mut Context) -> Response<State> {
        log::info!("State : StackInitialize - onEnter - coordinator : {}", context.is_coordinator);
        let band = TAdpBand::try_from_primitive(context.settings.g3.band).unwrap();
        let request = request::AdpInitializeRequest::from_band(&band);
        match context.usi_tx.send(usi::Message::UsiOut(request.into())) {
            Ok(_) => {
                Response::Handled
            },
            Err(e) => {
                log::warn!("Initialize Modem failed to send request : {}", e);
                Response::Transition(State::Idle) //TODO send to failed and recovery
            },
        }        
    }

    fn on_event(&mut self, event: &Message, context: &mut Context) -> Response<State> {
        log::trace!("StackInitialize : {:?}", event);
        match event {
            Message::Adp(adp) => {
                match adp {
                    adp::Message::AdpG3MsgStatusResponse(status_response) => {
                       //TODO check if success
                       Response::Transition(State::SetParams)
                    }
                    _ => {
                        Response::Handled
                    }
                }
            },
            _ => {
                Response::Handled
            }
        }        
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

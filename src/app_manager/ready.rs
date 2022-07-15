use crate::{app_config, usi, request::{AdpSetRequest, AdpInitializeRequest, self}, app_manager::Idle, adp};

use super::{Stateful, Response, State, Message};

pub struct Ready {

}

impl Ready {
    pub fn new() -> Self {
        
        Ready {
            
        }
    }
    
}

impl Stateful<State, usi::Message, flume::Sender<usi::Message>> for Ready {
    fn on_enter(&mut self, cs: &flume::Sender<usi::Message>) -> Response<State> {
        log::info!("State : Ready - onEnter");
        Response::Handled
    }

    fn on_event(&mut self, cs: &flume::Sender<usi::Message>, event: &Message) -> Response<State> {
        log::trace!("Ready : {:?}", event);
        match event {
            Message::Adp(adp) => {
                match adp {
                    adp::Message::AdpG3MsgStatusResponse(status_response) => {
                      Response::Handled
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

    fn on_exit(&mut self) {}
}

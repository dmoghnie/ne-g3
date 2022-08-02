use crate::{usi, request::AdpSetRequest, adp::{EAdpPibAttribute, self}};

use super::{Stateful, State, Context, Response, Message};


pub struct Idle {}
impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for Idle {
    fn on_enter(&mut self, cs: &flume::Sender<usi::Message>, context: &mut Context) -> Response<State> {

        Response::Handled
    }

    fn on_event(&mut self, cs: &flume::Sender<usi::Message>, event: &Message, context: &mut Context) -> Response<State> {
        log::info!("on event {:?}", event );
        match event {
            Message::Adp(event) => Response::Handled,
            Message::HeartBeat(time) => Response::Handled,
            Message::Startup => {
                // if context.is_coordinator {
                //     Response::Transition(State::SetCoordShortAddr)
                // }
                // else{
                //     Response::Transition(State::StackInitialize)
                // }
                Response::Transition(State::StackInitialize)
            },
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}
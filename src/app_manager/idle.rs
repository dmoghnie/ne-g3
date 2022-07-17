use crate::usi;

use super::{Stateful, State, Context, Response, Message};


pub struct Idle {}
impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for Idle {
    fn on_enter(&mut self, cs: &flume::Sender<usi::Message>, context: &mut Context) -> Response<State> {
        Response::Handled
    }

    fn on_event(&mut self, cs: &flume::Sender<usi::Message>, event: &Message, context: &mut Context) -> Response<State> {
        match event {
            Message::Adp(event) => Response::Handled,
            Message::HeartBeat(time) => Response::Handled,
            Message::Startup => Response::Transition(State::StackInitialize),
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}
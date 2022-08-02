use rand::Rng;

use crate::{usi, request::{AdpSetRequest, self}, adp::{EAdpPibAttribute, self}};

use super::{Stateful, State, Context, Response, Message};


pub struct SetCoordShortAddr {}
impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for SetCoordShortAddr {
    fn on_enter(&mut self, cs: &flume::Sender<usi::Message>, context: &mut Context) -> Response<State> {

        // let short_addr: u16 = rand::thread_rng().gen();
        let short_addr = 0u16;
        let v = short_addr.to_be_bytes().to_vec();
        let cmd = request::AdpMacSetRequest::new(adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS, 0, &v);
        cs.send(usi::Message::UsiOut(cmd.into()));
        Response::Handled
    }

    fn on_event(&mut self, cs: &flume::Sender<usi::Message>, event: &Message, context: &mut Context) -> Response<State> {
        log::info!("on event {:?}", event );
        match event {
            Message::Adp(event) => {
                match event {
                    adp::Message::AdpG3SetMacResponse(response) => {
                        //TODO check if success
                        Response::Transition(State::StackInitialize)
                     }
                     _ => {
                         Response::Handled
                     }
                }
            },
            Message::HeartBeat(time) => Response::Handled,
           _ => {Response::Handled}
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}
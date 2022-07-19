use crate::{usi, request::AdpSetRequest, adp::{EAdpPibAttribute, self}};

use super::{Stateful, State, Context, Response, Message};


pub struct Idle {}
impl Stateful<State, usi::Message, flume::Sender<usi::Message>, Context> for Idle {
    fn on_enter(&mut self, cs: &flume::Sender<usi::Message>, context: &mut Context) -> Response<State> {
        let v = vec![0u8, 1u8];
        let request = AdpSetRequest::new(adp::EAdpPibAttribute::ADP_IB_MANUF_IPV6_ULA_DEST_SHORT_ADDRESS, 
            0, &v);
        // let s = vec![0x0Au8];
        // let request = AdpSetRequest::new(adp::EAdpPibAttribute::ADP_IB_MAX_HOPS,
        //     0,
        //     &s);
        let v = request.into();
        log::info!("sending MANUF_IPV6_ULA_DEST {:?}", v);
        cs.send(usi::Message::UsiOut(v));
        Response::Handled
    }

    fn on_event(&mut self, cs: &flume::Sender<usi::Message>, event: &Message, context: &mut Context) -> Response<State> {
        log::info!("on event {:?}", event );
        match event {
            Message::Adp(event) => Response::Handled,
            Message::HeartBeat(time) => Response::Handled,
            Message::Startup => Response::Transition(State::StackInitialize),
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}
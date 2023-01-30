use nefsm::{Stateful, Response};

use crate::{usi, request::{AdpSetRequest, self}, adp::{EAdpPibAttribute, self, EAdpStatus}};

use super::{State, Context, Message};


pub struct DiscoverNetwork {}
impl Stateful<State, Context, Message<'_>> for DiscoverNetwork {
    fn on_enter(&mut self, context: &mut Context) -> Response<State> {

        log::info!("State : DiscoverNetwork - onEnter : context {:?}", context);

        let cmd = request::AdpDiscoveryRequest::new(context.settings.g3.discovery_timeout_secs);
        if let Err(e) = context.usi_tx.send(usi::Message::UsiOut(cmd.into())) {
            log::warn!("Failed to send network discovery request {}", e);
        }
        Response::Handled
    }

    fn on_event(&mut self, event: &Message, context: &mut Context) -> Response<State> {
        log::info!("on event {:?}", event );
        match event {
            Message::Adp(event) => {
                match event {
                    adp::Message::AdpG3DiscoveryResponse(response) => {
                        if response.status != EAdpStatus::G3_SUCCESS {
                            Response::Transition(State::NetworkDiscoverFailed)
                        }
                        else {
                            Response::Transition(State::JoinNetwork)
                        }
                    }
                    adp::Message::AdpG3DiscoveryEvent(event) => {
                        context.pan_descriptors.push(event.pan_descriptor.clone());
                        Response::Handled
                    }
                    _=>{
                        Response::Handled
                    }
                }
            },
            Message::HeartBeat(time) => Response::Handled,
            _ => {
                Response::Handled
            }
        }
    }

    fn on_exit(&mut self, context: &mut Context) {}
}
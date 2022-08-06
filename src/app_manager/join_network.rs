use crate::{usi, app_config, request, adp::{EAdpStatus, self, EMacWrpPibAttribute}};

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
        log::info!("State : JoinNetwork - onEnter : context {:?}", context);

        if let Some(ref pan_descriptor) = context.pan_descriptor {
            let cmd = request::AdpJoinNetworkRequest {
                pan_id: pan_descriptor.pan_id,
                lba_address: pan_descriptor.lba_address
            };
            if let Err(e) = cs.send(usi::Message::UsiOut(cmd.into())) {
                log::warn!("Failed to send network join request {}", e);
            }
    
        }
        else{
            log::error!("Trying to join network without pan descriptor");
        } //TODO handle when no pan descriptor
        Response::Handled
    }

    fn on_event(
        &mut self,
        cs: &flume::Sender<usi::Message>,
        event: &Message,
        context: &mut Context,
    ) -> Response<State> {
        match event {
            Message::Adp(adp) => {
                match adp {
                    adp::Message::AdpG3NetworkJoinResponse(response) => {
                        if (response.status == EAdpStatus::G3_SUCCESS) {
                            let v = response.network_addr.to_le_bytes().to_vec();
                            // let request = request::AdpMacSetRequest::new(EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS, 0, &v);
                            // cs.send(usi::Message::UsiOut(request.into()));
                            return Response::Transition(State::Idle);
                        }
                        else if (response.status == EAdpStatus::G3_TIMEOUT) {
                            return Response::Transition(State::JoinNetworkTimeout);
                        }
                    }
                    _ => {}
                }
            },
            _ => {}
        }
        Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {}
}

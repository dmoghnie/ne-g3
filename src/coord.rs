use std::io;

use crate::app_config;
use crate::lbp;
use crate::{
    adp,
    adp::{AdpG3, EAdpStatus, Message},
    common,
    common::Parameter,
    lbp::JoiningMessage,
    lbp_manager, request,
    usi::{self, MessageHandler},
};
use bytes::BytesMut;
use lazy_static::lazy_static;
use log;
use std::collections::HashMap;
use packet::ip::v6::Packet;

#[derive(thiserror::Error, Debug)]
enum CoordError {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    #[error("Failed to send cmd using channel")]
    SendError(#[source] flume::SendError<usi::Message>),

    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    Start,
    StackIntializing,
    SettingParameters,
    StartingNetwork,
    Ready,
}

lazy_static! {
    static ref MAC_STACK_PARAMETERS: Vec<(adp::EMacWrpPibAttribute, u16, Vec<u8>)> = vec![
        (
            adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS,
            0,
            vec![0x0, 0x0]
        ),
        (
            adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID,
            0,
            app_config::PAN_ID.to_be_bytes().to_vec()
        )
    ];
    static ref ADP_STACK_PARAMETERS: Vec<(adp::EAdpPibAttribute, u16, Vec<u8>)> = vec![
        (adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL, 0, vec![0x05]),

        (
            adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME,
            0,
            vec![0x00, 0x5A]
        ),
        (adp::EAdpPibAttribute::ADP_IB_MAX_HOPS, 0, vec![0x0A]),
        (adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY, 0, app_config::CONF_PSK_KEY.to_vec()),
        (adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 0, app_config::CONF_CONTEXT_INFORMATION_TABLE_0.to_vec()),
        // (adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 1, app_config::CONF_CONTEXT_INFORMATION_TABLE_1.to_vec()),
        (
            adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL,
            0,
            vec![0x0]
        ),
        (
            adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL,
            0,
            vec![0xB4, 0x00]
        ),


    ];
}

pub struct Coordinator {
    cmd_tx: flume::Sender<usi::Message>,
    net_tx: flume::Sender<adp::Message>,
    state: State,
    adp_param_idx: usize,
    mac_param_idx: usize,
    lbp_manager: lbp_manager::LbpManager,
}

impl MessageHandler for Coordinator {
    fn process(&mut self, msg: usi::Message) -> bool {
        log::trace!("->Coord : state ({:?}) : msg({:?})", self.state, msg);

        match msg {
            usi::Message::UsiIn(ref msg) => {
                self.process_usi_in_message(msg);
            }
            usi::Message::UsiOut(_) => {}
            usi::Message::HeartBeat(time) => {
                log::trace!("Adp received heartbeat {:?}", time);
                // if self.state == State::Ready{
                //     self.joinNetwork();
                // }
            }
            usi::Message::SystemStartup => {
                self.init();
            }
            
            _ => {

            }
        }
        log::trace!("<-Coord : state ({:?})", self.state);
        return true;
    }
}

impl Coordinator {
    pub fn new(cmd_tx: flume::Sender<usi::Message>, net_tx: flume::Sender<adp::Message>) -> Self {
        Coordinator {
            cmd_tx: cmd_tx,       
            net_tx: net_tx,     
            state: State::Start,
            adp_param_idx: 0,
            mac_param_idx: 0,
            lbp_manager: lbp_manager::LbpManager::new(),
        }
    }

    fn init(&mut self) {
        if (self.state != State::Start) {
            log::warn!("Received init in a non start state");
            return;
        }

        self.initializeStack();
    }

    fn process_usi_in_message(&mut self, msg: &usi::InMessage) {
        if let Some(msg) = adp::usi_message_to_message(&msg) {
            log::trace!(
                "process_usi_in_message: state {:?}: msg: {:?}",
                self.state,
                msg
            );
            match self.state {
                State::StackIntializing => {
                    self.process_state_stack_initializing(&msg);
                }
                State::SettingParameters => {
                    self.process_state_setting_parameters(&msg);
                }
                State::StartingNetwork => {
                    self.process_state_starting_network(&msg);
                }
                State::Ready => {
                    self.process_state_ready(msg);
                }
                _ => {
                    log::warn!("Received a message in an invalid state");
                }
            }
        } else {
            log::warn!("Failed to parse usi message: {:?}", msg);
        }
    }

    fn process_state_stack_initializing(&mut self, msg: &adp::Message) {
        match msg {
            Message::AdpG3MsgStatusResponse(status_response) => {
                self.setParameters();
            }
            _ => {}
        }
    }
    fn process_state_starting_network(&mut self, msg: &adp::Message) {
        match msg {
            Message::AdpG3NetworkStartResponse(start_response) => {
                if start_response.status == EAdpStatus::G3_SUCCESS {
                    self.state = State::Ready;
                } else {
                    log::warn!("Failed to start network"); // TODO, recovery
                }
            }
            _ => {}
        }
    }
    fn process_state_ready(&mut self, msg: adp::Message) {
        match msg {
            Message::AdpG3LbpEvent(lbp_event) => {
                if let Some(lbp_message) = lbp::adp_message_to_lbp_message(lbp_event) {
                    log::trace!("Received lbp_event {:?}", lbp_message);
                    if let Some(result) = self.lbp_manager.process_msg(&lbp_message) {
                        self.send_cmd(result.into());
                    }
                }
            }
            Message::AdpG3LbpReponse(lbp_response) => {
                self.lbp_manager.process_response (&lbp_response);
            }
            _ => {

            }
        }
    }
    fn process_state_setting_parameters(&mut self, msg: &adp::Message) {
        match msg {
            Message::AdpG3SetMacResponse(_) => {
                self.mac_param_idx = self.mac_param_idx + 1;
            }
            Message::AdpG3SetResponse(_) => {
                self.adp_param_idx = self.adp_param_idx + 1;
            }
            _ => {}
        }
        log::trace!(
            "process_state_setting_parameters: mac index : {}, adp index : {} : msg: {:?}",
            self.mac_param_idx,
            self.adp_param_idx,
            msg
        );
        if (self.adp_param_idx + self.mac_param_idx)
            >= (MAC_STACK_PARAMETERS.len() + ADP_STACK_PARAMETERS.len())
        {
            self.startNetwork();
        } else {
            self.setParameters();
        }
    }
    fn initializeStack(&mut self) {
        self.state = State::StackIntializing;
        let cmd = request::AdpInitializeRequest::from_band(app_config::BAND);
        match self.send_cmd(cmd.into()) {
            Err(e) => {
                //TODO, retry ?!
                log::warn!("Failed to initialize stack : {}", e);
            }
            _ => {}
        }
    }
    fn setParameters(&mut self) {
        log::trace!(
            "setParameters: mac index : {}, adp index : {}",
            self.mac_param_idx,
            self.adp_param_idx
        );
        self.state = State::SettingParameters;

        if self.mac_param_idx < MAC_STACK_PARAMETERS.len() {
            let attribute = &MAC_STACK_PARAMETERS[self.mac_param_idx];
            let cmd = request::AdpMacSetRequest::new(attribute.0, attribute.1, &attribute.2);
            if let Err(e) = self.send_cmd(cmd.into()) {
                log::warn!("Failed to send set parameter {:?}", attribute.0);
            }
        } else if self.adp_param_idx < ADP_STACK_PARAMETERS.len() {
            let attribute = &ADP_STACK_PARAMETERS[self.adp_param_idx];
            let cmd = request::AdpSetRequest::new(attribute.0, attribute.1, &attribute.2);
            if let Err(e) = self.send_cmd(cmd.into()) {
                log::warn!("Failed to send set parameter {:?}", attribute.0);
            }
        }
    }
    fn startNetwork(&mut self) {
        self.state = State::StartingNetwork;
        let cmd = request::AdpNetworkStartRequest::new(*app_config::PAN_ID);
        if let Err(e) = self.send_cmd(cmd.into()) {
            log::warn!("Failed to send network start request {}", e);
        }
    }
    fn joinNetwork(&mut self) {
        // self.state = State::JoiningNetwork;
        let cmd = request::AdpJoinNetworkRequest {
            pan_id: *app_config::PAN_ID,
            lba_address: 0,
        };
        if let Err(e) = self.send_cmd(cmd.into()) {
            log::warn!("Failed to send network start request {}", e);
        }
    }

    fn send_cmd(&self, msg: usi::OutMessage) -> Result<(), CoordError> {
        match self.cmd_tx.send(usi::Message::UsiOut(msg)) {
            Ok(_) => Ok(()),
            Err(e) => Err(CoordError::SendError(e)),
        }
    }
}

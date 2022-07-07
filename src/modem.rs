use std::{borrow::Borrow, io};

use crate::app_config;
use crate::{
    adp,
    adp::{EAdpStatus, Message},
    common,
    common::Parameter,
    network_manager::NetworkManager,
    request,
    usi::{self, MessageHandler},
};

use lazy_static::lazy_static;
use log;
use std::collections::HashMap;


#[derive(thiserror::Error, Debug)]
enum ModemError {
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
    JoiningNetwork,
    Ready,
    SendingData,
}

lazy_static! {

    static ref MAC_STACK_PARAMETERS: Vec<(adp::EMacWrpPibAttribute, u16, Vec<u8>)> = vec![
    //     (
    //     adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS,
    //     0,
    //     app_config::SENDER.to_vec()
    //     )
    // ,
    (adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID, 0, app_config::PAN_ID.to_be_bytes().to_vec())];
    static ref ADP_STACK_PARAMETERS: Vec<(adp::EAdpPibAttribute, u16, Vec<u8>)> = vec![
        (adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY, 0, app_config::CONF_PSK_KEY.to_vec()),
        (adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 0, app_config::CONF_CONTEXT_INFORMATION_TABLE_0.to_vec()),
        (adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 1, app_config::CONF_CONTEXT_INFORMATION_TABLE_1.to_vec()),
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
        (
            adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME,
            0,
            vec![0x00, 0x0f]
        ),
        (
            adp::EAdpPibAttribute::ADP_IB_MAX_HOPS,
            0,
            vec![0x0A],
        )
    ];
}

pub struct Modem {
    cmd_tx: flume::Sender<usi::Message>,
    net_tx: flume::Sender<adp::Message>,
    state: State,
    adp_param_idx: usize,
    mac_param_idx: usize,
    handle: u8,
}

impl MessageHandler for Modem {
    fn process(&mut self, msg: usi::Message) -> bool {
        log::info!("->Modem : state ({:?}) : msg({:?})", self.state, msg);

        match msg {
            usi::Message::UsiIn(ref msg) => {
                self.process_usi_in_message(msg);
            }
            usi::Message::UsiOut(_) => {}
            usi::Message::HeartBeat(time) => {
                log::info!(
                    "Adp received heartbeat: {:?}, state: {:?}",
                    time,
                    self.state
                );
                // if self.state == State::Ready {
                //     self.sendData();
                // }
                // self.sendData();
                // if self.state == State::JoiningNetwork {
                //     self.joinNetwork();
                // }
            }
            usi::Message::SystemStartup => {
                self.init();
            }
        }
        log::info!("<-Modem : state ({:?}) : msg({:?})", self.state, msg);
        return true;
    }
}

impl Modem {
    pub fn new(cmd_tx: flume::Sender<usi::Message>, net_tx: flume::Sender<adp::Message>) -> Self {
        Modem {
            cmd_tx: cmd_tx,
            net_tx: net_tx,
            state: State::Start,
            adp_param_idx: 0,
            mac_param_idx: 0,
            handle: 0,
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
            log::debug!(
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
                State::JoiningNetwork => {
                    self.process_state_joining_network(&msg);
                }
                State::SendingData => {
                    self.process_state_sending_data(&msg);
                }
                State::Ready => {}
                _ => {
                    log::warn!("Received a message in an invalid state");
                }
            }
            match self.net_tx.send(msg){
                Ok(_) => {
                    log::debug!("sent to network manager ");
                },
                Err(e) => {
                    log::warn!("Error sending to network manager {:?}", e);
                },
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
    fn process_state_joining_network(&mut self, msg: &adp::Message) {
        match msg {
            Message::AdpG3NetworkJoinResponse(join_network_response) => {
                match join_network_response.status {
                    EAdpStatus::G3_SUCCESS => {
                        self.state = State::Ready;
                    },
                    EAdpStatus::G3_TIMEOUT => {
                        self.joinNetwork();
                    }
                    _ => {
                        log::warn!("Failed to join network : {:?}", join_network_response.status); // TODO, recovery
                    }
                }
            },
            Message::AdpG3SetMacResponse(mac_set_response) => {
                log::info!("Mac Set response {:?}", mac_set_response);
                self.state = State::Ready;
            }
            _ => {}
        }
    }
    fn process_ready_state(&mut self, msg: &adp::Message) {}
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
        log::info!(
            "process_state_setting_parameters: mac index : {}, adp index : {} : msg: {:?}",
            self.mac_param_idx,
            self.adp_param_idx,
            msg
        );
        if (self.adp_param_idx + self.mac_param_idx)
            >= (MAC_STACK_PARAMETERS.len() + ADP_STACK_PARAMETERS.len())
        {
            self.joinNetwork();
        } else {
            self.setParameters();
        }
    }
    fn process_state_sending_data(&mut self, msg: &adp::Message) {
        // self.sendData();
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
    fn set_short_addr(&self, short_addr: u16) {
        let v = short_addr.to_be_bytes().to_vec();
        let cmd = request::AdpMacSetRequest::new(
            adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS,
            0,
            &v,
        );
        if let Err(e) = self.send_cmd(cmd.into()) {
            log::warn!("Failed to send parameter MAC_WRP_PIB_SHORT_ADDRESS : {:?} ", e);
        }
    }
    fn setParameters(&mut self) {
        log::info!(
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
    fn joinNetwork(&mut self) {
        // let get_address_request = request::AdpMacGetRequest::new(adp::EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS, 0);
        // self.send_cmd(get_address_request.into());
        self.state = State::JoiningNetwork;
        let cmd = request::AdpJoinNetworkRequest {
            pan_id: *app_config::PAN_ID,
            lba_address: 0,
        };
        if let Err(e) = self.send_cmd(cmd.into()) {
            log::warn!("Failed to send network start request {}", e);
        }
    }

    // fn sendData(&mut self) {
    //     self.state = State::SendingData;
    //     let receiver = app_config::RECEIVER.to_vec();
    //     let cmd1 = request::AdpSetRequest::new(
    //         adp::EAdpPibAttribute::ADP_IB_DESTINATION_ADDRESS_SET,
    //         0,
    //         &receiver,
    //     );
    //     self.send_cmd(cmd1.into());
    //     self.handle = self.handle + 1;
    //     let cmd = request::AdpDataRequest::new(self.handle, &vec![1, 2, 3, 4, 5, 6], true, 0);
    //     self.send_cmd(cmd.into());
    // }

    fn send_cmd(&self, msg: usi::OutMessage) -> Result<(), ModemError> {
        match self.cmd_tx.send(usi::Message::UsiOut(msg)) {
            Ok(_) => Ok(()),
            Err(e) => Err(ModemError::SendError(e)),
        }
    }
}

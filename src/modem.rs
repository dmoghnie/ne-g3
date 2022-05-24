use std::{io, borrow::Borrow};

use crate::{adp, adp::{Message, EAdpStatus}, common, common::Parameter, request, usi::{self, MessageHandler}};
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
    SendingData
}
const PAN_ID:u16 = 0x781D;
const SENDER:[u8;2] = [0x0, 0x1];
const RECEIVER:[u8;2] = [0x0, 0x2];

const CONF_PSK_KEY:[u8; 16] = [0xab, 0x10, 0x34, 0x11, 0x45, 0x11, 0x1b, 0xc3, 0xc1, 0x2d, 0xe8, 0xff, 0x11, 0x14, 0x22, 0x4];
const CONF_GMK_KEY:[u8; 16] = [0xaf, 0x4d, 0x6d, 0xcc, 0xf1, 0x4d, 0xe7, 0xc1, 0xc4, 0x23, 0x5e, 0x6f, 0xef, 0x6c, 0x15, 0x1f];
const CONF_CONTEXT_INFORMATION_TABLE_0:[u8;14] = [0x2, 0x0, 0x1, 0x50, 0xfe, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x78, 0x1d];
const CONF_CONTEXT_INFORMATION_TABLE_1:[u8;10] = [0x2, 0x0, 0x1, 0x30, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

lazy_static! {

    static ref MAC_STACK_PARAMETERS: Vec<(adp::EMacWrpPibAttribute, u16, Vec<u8>)> = vec![(        
        adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS,
        0,
        SENDER.to_vec()
    ), (adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID, 0, vec![0x78, 0x1d])];
    static ref ADP_STACK_PARAMETERS: Vec<(adp::EAdpPibAttribute, u16, Vec<u8>)> = vec![
        (adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY, 0, CONF_PSK_KEY.to_vec()),
        (adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 0, CONF_CONTEXT_INFORMATION_TABLE_0.to_vec()),
        (adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 1, CONF_CONTEXT_INFORMATION_TABLE_1.to_vec()),
        (
            adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL,
            0,
            vec![0x5]
        ),
        (
            adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL,
            0,
            vec![0xB4, 0x00]
        ),
        (        
            adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME,
            0,
            vec![0x1A, 0x00]
        ),
        (            
            adp::EAdpPibAttribute::ADP_IB_MAX_HOPS,
            0,
            vec![0x0A],
        )
    ];
}

const BAND: adp::TAdpBand = adp::TAdpBand::ADP_BAND_CENELEC_A;
pub struct Modem {
    cmd_tx: flume::Sender<usi::Message>,
    state: State,
    adp_param_idx: usize,
    mac_param_idx: usize,
    handle: u8
}

impl MessageHandler for Modem {
    fn process(&mut self, msg: usi::Message) -> bool {
        log::trace!("->Modem : state ({:?}) : msg({:?})", self.state, msg);

        match msg {
            usi::Message::UsiIn(ref msg) => {
                self.process_usi_in_message(msg);
            }
            usi::Message::UsiOut(_) => {}
            usi::Message::HeartBeat(time) => {
                log::trace!("Adp received heartbeat: {:?}, state: {:?}", time, self.state);
                if self.state == State::Ready {
                    self.sendData();
                }
                // self.sendData();
                // if self.state == State::JoiningNetwork {
                //     self.joinNetwork();
                // }
            }
            usi::Message::SystemStartup => {
                self.init();
            }
        }
        log::trace!("<-Modem : state ({:?}) : msg({:?})", self.state, msg);
        return true;
    }
}

impl Modem {
    pub fn new(cmd_tx: flume::Sender<usi::Message>) -> Self {
        Modem {
            cmd_tx: cmd_tx,
            state: State::Start,
            adp_param_idx: 0,
            mac_param_idx: 0,
            handle:0
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
            log::trace!("process_usi_in_message: state {:?}: msg: {:?}", self.state, msg);
            match self.state {
                State::StackIntializing => {
                    self.process_state_stack_initializing(&msg);
                },
                State::SettingParameters => {
                    self.process_state_setting_parameters(&msg);
                }
                State::JoiningNetwork => {
                    self.process_state_joining_network(&msg);
                },
                State::SendingData => {
                    self.process_state_sending_data (&msg);
                }
                State::Ready => {}
                _ => {
                    log::warn!("Received a message in an invalid state");
                }
            }
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
                if join_network_response.status == EAdpStatus::G3_SUCCESS {
                    self.state = State::Ready;
                }
                else{
                    log::warn!("Failed to join network"); // TODO, recovery
                }
            }
            _=>{}
        }
    }
    fn process_ready_state(&mut self, msg: &adp::Message) {}
    fn process_state_setting_parameters(&mut self, msg: &adp::Message) {


        match msg {
            Message::AdpG3SetMacResponse(_) => {
                self.mac_param_idx = self.mac_param_idx + 1;
            },
            Message::AdpG3SetResponse(_) => {
                self.adp_param_idx = self.adp_param_idx + 1;
            },
            _ => {}
        }
        log::trace!("process_state_setting_parameters: mac index : {}, adp index : {} : msg: {:?}", self.mac_param_idx, self.adp_param_idx, msg);
        if (self.adp_param_idx + self.mac_param_idx) >= (MAC_STACK_PARAMETERS.len() + ADP_STACK_PARAMETERS.len()) {
             self.joinNetwork();
        }
        else{
            self.setParameters();
        }
    }
    fn process_state_sending_data(&mut self, msg: &adp::Message){
        // self.sendData();
    }
    fn initializeStack(&mut self) {        
        self.state = State::StackIntializing;
        let cmd = request::AdpInitializeRequest::from_band(BAND);
        match self.send_cmd(cmd.into()) {
            Err(e) => {
                //TODO, retry ?!
                log::warn!("Failed to initialize stack : {}", e);
            }
            _ => {}
        }
    }
    fn setParameters(&mut self) {
        log::trace!("setParameters: mac index : {}, adp index : {}", self.mac_param_idx, self.adp_param_idx);
        self.state = State::SettingParameters;

        if self.mac_param_idx < MAC_STACK_PARAMETERS.len() {
            let attribute = &MAC_STACK_PARAMETERS[self.mac_param_idx];
            let cmd = request::AdpMacSetRequest::new (attribute.0, attribute.1, &attribute.2);
            if let Err(e) = self.send_cmd(cmd.into()){
                log::warn!("Failed to send set parameter {:?}", attribute.0);
            }
        } 
        else if self.adp_param_idx < ADP_STACK_PARAMETERS.len() {
            let attribute = &ADP_STACK_PARAMETERS[self.adp_param_idx];
            let cmd = request::AdpSetRequest::new (attribute.0, attribute.1, &attribute.2);
            if let Err(e) = self.send_cmd(cmd.into()){
                log::warn!("Failed to send set parameter {:?}", attribute.0);
            }
        }
    }
    fn joinNetwork(&mut self) {
        // let get_address_request = request::AdpMacGetRequest::new(adp::EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS, 0);                
        // self.send_cmd(get_address_request.into());
        self.state = State::JoiningNetwork;
        let cmd = request::AdpJoinNetworkRequest {pan_id: PAN_ID, lba_address: 0};
        if let Err(e) = self.send_cmd(cmd.into()) {
            log::warn!("Failed to send network start request {}", e);
        }
    }
    
    fn sendData(&mut self) {
        self.state = State::SendingData;
        let receiver = RECEIVER.to_vec();
        let cmd1 = request::AdpSetRequest::new (adp::EAdpPibAttribute::ADP_IB_DESTINATION_ADDRESS_SET, 0, 
            &receiver);
        self.send_cmd (cmd1.into());
        self.handle = self.handle + 1;
        let cmd = request::AdpDataRequest::new (self.handle, &vec![1,2,3,4,5,6], true, 0);
        self.send_cmd (cmd.into());
    }

    fn send_cmd(&self, msg: usi::OutMessage) -> Result<(), ModemError> {
        match self.cmd_tx.send(usi::Message::UsiOut(msg)) {
            Ok(_) => Ok(()),
            Err(e) => Err(ModemError::SendError(e)),
        }
    }
}

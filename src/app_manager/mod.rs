use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::thread::current;
use std::time::SystemTime;

use flume;
use flume::SendError;
use nefsm::StateMachine;
use serialport::new;

use crate::adp;

use crate::adp::TAdpPanDescriptor;
use crate::adp::TExtendedAddress;
use crate::app_config;
use crate::app_manager::ready::Ready;
use crate::app_manager::set_params::SetParams;
use crate::app_manager::stack_initialize::StackInitialize;
use crate::lbp;
use crate::lbp_manager;
use crate::request::AdpMacSetRequest;
use crate::request::AdpSetRequest;
use crate::usi;

use self::get_params::GetParams;
use self::idle::Idle;
use self::join_network::JoinNetwork;
use self::join_network_failed::JoinNetworkFailed;
use self::network_discover_failed::NetworkDiscoverFailed;
use self::set_coord_short_addr::SetCoordShortAddr;
use self::start_network::StartNetwork;
use self::discover_network::DiscoverNetwork;

pub mod stack_initialize;
pub mod ready;
pub mod set_params;
pub mod join_network;
pub mod idle;
pub mod get_params;
pub mod start_network;
pub mod set_coord_short_addr;
pub mod join_network_failed;
pub mod discover_network;
pub mod network_discover_failed;

use nefsm::{Stateful, FsmEnum};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum State {
    Idle,
    SetCoordShortAddr,
    StackInitialize,
    SetParams,
    GetParams,
    JoinNetwork,
    StartNetwork,
    Ready,
    JoinNetworkFailed,
    DiscoverNetwork,
    NetworkDiscoverFailed,
}


impl <'a> FsmEnum<State, Context, Message> for State {
    fn create(enum_value: &State) -> Box<dyn Stateful<State, Context, Message> + Send> {
        match enum_value {
            State::Idle => Box::new(Idle {}),
            State::SetCoordShortAddr => Box::new(SetCoordShortAddr {}),
            State::StackInitialize => Box::new(StackInitialize {}),
            State::SetParams => Box::new(SetParams::new()),
            State::GetParams => Box::new(GetParams {}),
            State::JoinNetwork => Box::new(JoinNetwork {}),
            State::StartNetwork => Box::new(StartNetwork {}),
            State::Ready => Box::new(Ready {}),
            State::JoinNetworkFailed => Box::new(JoinNetworkFailed {}),
            State::DiscoverNetwork => Box::new(DiscoverNetwork {}),
            State::NetworkDiscoverFailed => Box::new(NetworkDiscoverFailed {}),
        }    
    }
}

#[derive(Debug)]
pub enum Message {
    Adp(adp::Message),
    HeartBeat(SystemTime),
    Startup,
}
pub trait CommandSender<C> {
    fn send_cmd(&self, cmd: C) -> bool;
}

#[derive(Debug)]
pub struct Context {
    is_coordinator: bool,
    extended_addr: Option<TExtendedAddress>,
    settings: app_config::Settings,
    pan_descriptors: Vec<TAdpPanDescriptor>,
    usi_tx: flume::Sender<usi::Message>,
   
}

pub struct AppManager {
    usi_tx: flume::Sender<usi::Message>,
    net_tx: flume::Sender<adp::Message>,
}

impl <'a>AppManager {
    pub fn new(
        usi_tx: flume::Sender<usi::Message>,
        net_tx: flume::Sender<adp::Message>,
        
    ) -> Self {
        AppManager {
            usi_tx,
            net_tx,
        }
    }
    
    // fn init_states( state_machine: &mut StateMachine::<State, usi::Message, Context>) {
    //     state_machine.add_state(State::Idle, Box::new(Idle {}));
    //     state_machine.add_state(State::StackInitialize, Box::new(StackInitialize::new()));
    //     state_machine.add_state(State::SetParams, Box::new(SetParams::new()));
    //     state_machine.add_state(State::GetParams, Box::new(GetParams::new()));
    //     state_machine.add_state(State::JoinNetwork, Box::new(JoinNetwork::new()));
    //     state_machine.add_state(State::StartNetwork, Box::new(StartNetwork::new()));
    //     state_machine.add_state(State::Ready, Box::new(Ready::new()));
    //     state_machine.add_state(State::JoinNetworkFailed, Box::new(JoinNetworkFailed {}));
    //     state_machine.add_state(State::DiscoverNetwork, Box::new(DiscoverNetwork {}));
    //     state_machine.add_state(State::SetCoordShortAddr, Box::new(SetCoordShortAddr {}));
    //     state_machine.add_state(State::NetworkDiscoverFailed, Box::new(NetworkDiscoverFailed {}));
    // }
    pub fn start(self, settings: &app_config::Settings,  usi_receiver: flume::Receiver<usi::Message>, is_coordinator: bool) {
        log::info!("App Manager started ...");
        let settings = settings.clone();
        let mut state_machine =
                StateMachine::<State, Context, Message>::new(
                    Context { is_coordinator, extended_addr: None, 
                        settings: settings, pan_descriptors: Vec::new(), usi_tx: self.usi_tx.clone() }
                );
        thread::spawn(move || {
            
                state_machine.init(State::Idle);
            // let mut lbp_manager = lbp_manager::LbpManager::new();
            // Self::init_states(&mut state_machine);
            
           
            loop {
                match usi_receiver.recv() {
                    Ok(event) => {
                        log::info!("app_manager - {:?} received msg : {:?}", state_machine.get_current_state(), event);
                        match event {
                            usi::Message::UsiIn(usi_msg) => {
                                if let Some(adp_msg) = adp::usi_message_to_message(&usi_msg){
                                    let m = Message::Adp(adp_msg);
                                    //TODO optimize, split event those needed by the state machine and those needed by network manager
                                    state_machine.process_event(&m);
                                    match m {
                                        Message::Adp(adp) => {
                                            if let Err(e) = self.net_tx.send(adp ) {
                                                log::warn!("Failed to send adp message to network manager {}", e);
                                            }        
                                        },
                                        _ =>{}
                                       
                                    }
                            
                            
                                }
                               
                            }

                            usi::Message::HeartBeat(time) => {
                                state_machine.process_event(&Message::HeartBeat(time));
                            }
                            usi::Message::SystemStartup => {
                                state_machine.process_event(&Message::Startup);
                            } 
                            _ => {}
                        }
                        
                    }
                    Err(e) => {
                        log::warn!("app_manager : failed to receive message {}", e)
                    }
                }
            }
        });
    }
}

impl CommandSender<usi::Message> for flume::Sender<usi::Message> {
    fn send_cmd(&self, cmd: usi::Message) -> bool {
        self.send(cmd);
        true
    }
}



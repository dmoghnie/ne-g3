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
#[nefsm_macro::fsm_trait(State, Context, Message<'_>)]
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
#[derive(Debug)]
pub enum Message<'a> {
    Adp(&'a adp::Message),
    HeartBeat(SystemTime),
    Startup,
}
pub trait CommandSender<C> {
    fn send_cmd(&self, cmd: C) -> bool;
}
// pub trait Stateful<S: Hash + PartialEq + Eq + Clone, C, CS: CommandSender<C>, CTX> {
//     fn on_enter(&mut self, cs: &CS, context: &mut CTX) -> Response<S>;
//     fn on_event(&mut self, cs: &CS, event: &Message, context: &mut CTX) -> Response<S>;
//     fn on_exit(&mut self, context: &mut CTX);
// }

// pub enum Response<S> {
//     Handled,
//     Transition(S),
// }
// pub enum Error<S> {
//     Handled,
//     Transition(S),
// }
// pub trait ResultExt<T, S> {
//     fn or_transition(self, state: S) -> core::result::Result<T, Error<S>>;

//     fn or_handle(self) -> core::result::Result<T, Error<S>>;
// }

// impl<T, E, S> ResultExt<T, S> for core::result::Result<T, E> {
//     fn or_transition(self, state: S) -> core::result::Result<T, Error<S>> {
//         self.map_err(|_| Error::Transition(state))
//     }

//     fn or_handle(self) -> core::result::Result<T, Error<S>> {
//         self.map_err(|_| Error::Handled)
//     }
// }
// impl<T, S> ResultExt<T, S> for core::option::Option<T> {
//     fn or_transition(self, state: S) -> core::result::Result<T, Error<S>> {
//         self.ok_or(Error::Transition(state))
//     }

//     fn or_handle(self) -> core::result::Result<T, Error<S>> {
//         self.ok_or(Error::Handled)
//     }
// }

// pub struct StateMachine<S: Hash + PartialEq + Eq + Clone, C, CS: CommandSender<C>, CTX> {
//     states: HashMap<S, Box<dyn Stateful<S, C, CS, CTX>>>,
//     current_state: S,
//     command_sender: CS,
//     context: CTX
// }
// impl<S: Hash + PartialEq + Eq + Clone, C, CS, CTX> StateMachine<S, C, CS, CTX>
// where
//     CS: CommandSender<C>, S: Debug, CTX: Sized
// {
//     pub fn new(initial_state: S, command_sender: CS, context: CTX) -> Self {
//         let mut states = HashMap::<S, Box<dyn Stateful<S, C, CS, CTX>>>::new();
//         Self {
//             states: states,
//             current_state: initial_state,
//             command_sender: command_sender,
//             context: context
//         }
//     }
//     pub fn add_state(&mut self, s: S, state: Box<dyn Stateful<S, C, CS, CTX>>) {
//         self.states.insert(s, state);
//     }

//     fn process_event(&mut self, event: &Message) {
//         let state = self.states.get_mut(&self.current_state);

//         if let Some(st) = state {
//             match st.on_event(&self.command_sender, event, &mut self.context) {
//                 Response::Handled => {}
//                 Response::Transition(s) => {
//                     if s != self.current_state {
//                         st.on_exit(&mut self.context);
//                         self.current_state = s;
//                         loop {
//                             log::info!("StateMachine : {:?} - {:?}", self.current_state, event);
//                             if let Some(s) = self.states.get_mut(&self.current_state) {
//                                 match s.on_enter(&self.command_sender, &mut self.context) {
//                                     Response::Handled => {
//                                         break;
//                                     }
//                                     Response::Transition(s) => {
//                                         if s == self.current_state {
//                                             break;
//                                         } else {
//                                             self.current_state = s;
//                                         }
//                                     }
//                                 }
//                             }
//                             else{
//                                 log::warn!("Failed to find state : {:?}", self.current_state);
//                                 break;
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     // pub fn on_enter(&mut self) {
//     //     if let Some(state) = self.states.get_mut(&self.current_state){
//     //         state.on_enter();
//     //     }
//     // }
//     // pub fn on_exit(&mut self) {
//     //     if let Some(state) = self.states.get_mut(&self.current_state){
//     //         state.on_exit();
//     //     }
//     // }
// }


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

impl AppManager {
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
        thread::spawn(move || {
            let mut state_machine =
                StateMachine::<State, Context, Message>::new(
                    Context { is_coordinator, extended_addr: None, 
                        settings: settings, pan_descriptors: Vec::new(), usi_tx: self.usi_tx.clone() }
                );
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
                                    //TODO optimize, split event those needed by the state machine and those needed by network manager
                                    state_machine.process_event(&Message::Adp(&adp_msg));
                                    if let Err(e) = self.net_tx.send(adp_msg) {
                                        log::warn!("Failed to send adp message to network manager {}", e);
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



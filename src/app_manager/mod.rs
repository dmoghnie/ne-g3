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

use crate::adp;
use crate::adp::usi_message_to_message;
use crate::app;
use crate::app_config;
use crate::app_manager::ready::Ready;
use crate::app_manager::set_params::SetParams;
use crate::app_manager::stack_initialize::StackInitialize;
use crate::request::AdpMacSetRequest;
use crate::request::AdpSetRequest;
use crate::usi;

mod stack_initialize;
mod ready;
mod set_params;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum State {
    Idle,
    StackInitialize,
    SetParams,
    Ready,
}
#[derive(Debug)]
enum Message {
    Adp(adp::Message),
    HeartBeat(SystemTime),
    Startup,
}
pub trait CommandSender<C> {
    fn send_cmd(&self, cmd: C) -> bool;
}
pub trait Stateful<S: Hash + PartialEq + Eq + Clone, C, CS: CommandSender<C>> {
    fn on_enter(&mut self, cs: &CS) -> Response<S>;
    fn on_event(&mut self, cs: &CS, event: &Message) -> Response<S>;
    fn on_exit(&mut self);
}

pub enum Response<S> {
    Handled,
    Transition(S),
}
pub enum Error<S> {
    Handled,
    Transition(S),
}
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

pub struct StateMachine<S: Hash + PartialEq + Eq + Clone, C, CS: CommandSender<C>> {
    states: HashMap<S, Box<dyn Stateful<S, C, CS>>>,
    current_state: S,
    command_sender: CS,
}
impl<S: Hash + PartialEq + Eq + Clone, C, CS> StateMachine<S, C, CS>
where
    CS: CommandSender<C>, S: Debug
{
    pub fn new(initial_state: S, command_sender: CS) -> Self {
        let mut states = HashMap::<S, Box<dyn Stateful<S, C, CS>>>::new();
        Self {
            states: states,
            current_state: initial_state,
            command_sender: command_sender,
        }
    }
    pub fn add_state(&mut self, s: S, state: Box<dyn Stateful<S, C, CS>>) {
        self.states.insert(s, state);
    }

    fn process_event(&mut self, event: &Message) {
        let state = self.states.get_mut(&self.current_state);

        if let Some(st) = state {
            match st.on_event(&self.command_sender, event) {
                Response::Handled => {}
                Response::Transition(s) => {
                    if s != self.current_state {
                        st.on_exit();
                        self.current_state = s;
                        loop {
                            log::info!("StateMachine : {:?} - {:?}", self.current_state, event);
                            if let Some(s) = self.states.get_mut(&self.current_state) {
                                match s.on_enter(&self.command_sender) {
                                    Response::Handled => {
                                        break;
                                    }
                                    Response::Transition(s) => {
                                        if s == self.current_state {
                                            break;
                                        } else {
                                            self.current_state = s;
                                        }
                                    }
                                }
                            }
                            else{
                                log::warn!("Failed to find state : {:?}", self.current_state);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    // pub fn on_enter(&mut self) {
    //     if let Some(state) = self.states.get_mut(&self.current_state){
    //         state.on_enter();
    //     }
    // }
    // pub fn on_exit(&mut self) {
    //     if let Some(state) = self.states.get_mut(&self.current_state){
    //         state.on_exit();
    //     }
    // }
}

struct Idle {}
impl Stateful<State, usi::Message, flume::Sender<usi::Message>> for Idle {
    fn on_enter(&mut self, cs: &flume::Sender<usi::Message>) -> Response<State> {
        Response::Handled
    }

    fn on_event(&mut self, cs: &flume::Sender<usi::Message>, event: &Message) -> Response<State> {
        match event {
            Message::Adp(event) => Response::Handled,
            Message::HeartBeat(time) => Response::Handled,
            Message::Startup => Response::Transition(State::StackInitialize),
        }
    }

    fn on_exit(&mut self) {}
}

pub struct AppManager {
    usi_tx: flume::Sender<usi::Message>,
    net_tx: flume::Sender<adp::Message>,
    is_coord: bool,
}

impl AppManager {
    pub fn new(
        usi_tx: flume::Sender<usi::Message>,
        net_tx: flume::Sender<adp::Message>,
        is_coord: bool,
    ) -> Self {
        AppManager {
            usi_tx,
            net_tx,
            is_coord,
        }
    }
    fn send_cmd(&self, msg: usi::OutMessage) -> Result<(), SendError<usi::Message>> {
        let result = self.usi_tx.send(usi::Message::UsiOut(msg));
        if let Err(e) = result {
            log::warn!("Send cmd: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }
    fn init_states( state_machine: &mut StateMachine::<State, usi::Message, flume::Sender<usi::Message>>) {
        state_machine.add_state(State::Idle, Box::new(Idle {}));
        state_machine.add_state(State::StackInitialize, Box::new(StackInitialize::new()));
        state_machine.add_state(State::SetParams, Box::new(SetParams::new()));
        state_machine.add_state(State::Ready, Box::new(Ready::new()));
    }
    pub fn start(self, usi_receiver: flume::Receiver<usi::Message>) {
        log::info!("App Manager started ...");
        thread::spawn(move || {
            let mut state_machine =
                StateMachine::<State, usi::Message, flume::Sender<usi::Message>>::new(
                    State::Idle,
                    self.usi_tx.clone(),
                );
            Self::init_states(&mut state_machine);
            
            let mut msg: Option<Message> = None;
            loop {
                match usi_receiver.recv() {
                    Ok(event) => {
                        log::info!("app_manager - {:?} received msg : {:?}", state_machine.current_state, event);
                        match event {
                            usi::Message::UsiIn(usi_msg) => {
                                msg = usi_message_to_message(&usi_msg)
                                    .map_or(None, |v| Some(Message::Adp(v)));
                            }

                            usi::Message::HeartBeat(time) => {
                                msg = Some(Message::HeartBeat(time));
                            }
                            usi::Message::SystemStartup => msg = Some(Message::Startup),
                            _ => {}
                        }
                        if let Some(app_msg) = &msg {
                            state_machine.process_event(app_msg);
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



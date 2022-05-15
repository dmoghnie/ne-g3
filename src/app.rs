use std::thread;

use crate::usi::{MessageType};


trait StateImpl : Send + Sync{
    fn on_enter(&self, app: &mut App);
    fn on_msg (&self, app: &mut App, msg:&MessageType) -> Option<&dyn StateImpl>;
    fn on_exit(&self, app: &mut App);
}

pub struct App<'a> {
    thread_handle: Option<thread::JoinHandle<()>>,
    state: Option<Box<&'a dyn StateImpl>>,    
    cmd_tx: Option<flume::Sender<&'a MessageType>>
}

impl <'a> App<'a>{
    pub fn new(cmd_tx: &flume::Sender<MessageType>) -> Self {
        App {
            thread_handle: None,
            state: None,            
            cmd_tx: None,
        }
    }
    pub fn process_msg(&mut self, msg: &MessageType){
        if let Some(s) = &self.state {
            s.on_msg (self, msg);
        }
    }
}


#[derive(PartialEq, Eq)]
struct Start {

}
impl StateImpl for Start {
    fn on_enter(&self, app: &mut App) {
        
    }

    fn on_msg (&self, app: &mut App, msg:&MessageType) -> Option<&dyn StateImpl> {
        todo!()
    }

    fn on_exit(&self, app: &mut App) {
        todo!()
    }
}
#[derive(PartialEq, Eq)]
struct Idle {

}
impl StateImpl for Idle {
    fn on_enter(&self, app: &mut App) {
        todo!()
    }

    fn on_msg (&self, app: &mut App, msg:&MessageType) -> Option<&dyn StateImpl> {
        todo!()
    }

    fn on_exit(&self, app: &mut App) {
        todo!()
    }
}
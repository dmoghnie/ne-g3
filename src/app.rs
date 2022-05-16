use std::thread;

use serialport::Error;

use crate::{usi::{MessageType, UsiCommand}, request::{AdpInitializeRequest, AdpDiscoveryRequest, AdpGetRequest}, message, usi};


trait StateImpl : Send + Sync{
    fn get_name(&self)->&str;
    fn on_enter(&self, app: &App);
    fn on_msg (&self, app: &App, msg:&message::Message) -> Option<&dyn StateImpl>;
    fn on_exit(&self, app: &App);
}

pub struct App<'a> {    
    state: Option<Box<&'a dyn StateImpl>>,    
    cmd_tx: &'a flume::Sender<MessageType>
}

impl <'a> App<'a>{
    pub fn new(cmd_tx: &'a flume::Sender<MessageType>) -> Self {
        App {
            state: None,            
            cmd_tx: cmd_tx,
        }
    }
    pub fn init(&mut self) {
        self.state = Some(Box::new(&Start {}));
        let cmd = AdpInitializeRequest::from_band(message::TAdpBand::ADP_BAND_CENELEC_A);	//TODO parameterize

        if let Ok(c) = cmd.try_into() {
            self.cmd_tx.send (usi::MessageType::UsiCommand(c));
        }        
    }
    pub fn process_msg(&mut self, msg: &MessageType) -> bool{
        log::trace!("App processing message {:?}", msg);
        if let Some(s) = &self.state {            
            match msg {
                MessageType::UsiMessage(msg) => {
                    if let Some(m) = message::usi_message_to_message (msg){
                        let new_state = s.on_msg(self,&m);
                        if new_state.is_none() {
                            log::trace!("State {}, on exit", s.get_name());
                            s.on_exit(self);
                            return false;
                        }                
                        let new_state = new_state.unwrap();       
                        if new_state.get_name() != s.get_name() {
                            log::trace!("State {}, on exit", s.get_name());
                            s.on_exit(self);
                            self.state = Some(Box::new(new_state));
                            log::trace!("State {}, on enter", new_state.get_name());
                            new_state.on_enter(self);

                        }
                    }
                    else{
                        log::warn!("Adp failed to process message ...");
                    }
    
                }
                MessageType::UsiCommand(_) => {
                    log::warn!("App received a UsiCommand");
                }, 
            }            
        }    
        return true;    
    }
}


#[derive(PartialEq, Eq)]
struct Start {
    
}
impl StateImpl for Start {

    fn on_enter(&self, app: &App) {
        
    }

    fn on_msg (&self, app: &App, msg:&message::Message) -> Option<&dyn StateImpl> {
        log::trace!("received message : {:?}", msg);
        return Some(&Idle{});
    }

    fn on_exit(&self, app: &App) {
        
    }

    fn get_name(&self)->&str {
        "Start"
    }
}
#[derive(PartialEq, Eq)]
struct Idle {

}
impl StateImpl for Idle {
    fn on_enter(&self, app: &App) {
        let discovery = AdpDiscoveryRequest::new(2);
        if let Ok(c) = discovery.try_into() {
            app.cmd_tx.send (usi::MessageType::UsiCommand(c));
        }   
    }

    fn on_msg (&self, app: &App, msg:&message::Message) -> Option<&dyn StateImpl> {
        log::trace!("received message : {:?}", msg);
        return Some(&GetVersion{});
    }

    fn on_exit(&self, app: &App) {
        
    }

    fn get_name(&self)->&str {
        "Idle"
    }
}

struct GetVersion {}

impl StateImpl for GetVersion {
    fn get_name(&self)->&str {
        "GetVersion"
    }

    fn on_enter(&self, app: &App) {
        let get_version = AdpGetRequest::new (message::EAdpPibAttribute::ADP_IB_SOFT_VERSION, 0);
        if let Ok(c) = get_version.try_into() {
            app.cmd_tx.send (usi::MessageType::UsiCommand(c));
        }  
    }

    fn on_msg (&self, app: &App, msg:&message::Message) -> Option<&dyn StateImpl> {
        log::trace!("state {}, msg {:?}", self.get_name(), msg);
        None
    }

    fn on_exit(&self, app: &App) {
        
    }
}
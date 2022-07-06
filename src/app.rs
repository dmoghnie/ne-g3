use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;

use serialport::new;
use serialport::Error;

use crate::adp;
use crate::common;
use crate::lbp;
use crate::request;
use crate::usi;
use num_enum::*;

const SCAN_DURATION: u8 = 20;
const PAN_ID: u16 = 0x7812;

trait StateImpl {
    fn get_name(&self) -> &str;
    fn on_enter(&self, app: &App);
    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>>;
    fn on_exit(&self, app: &App);
}

pub struct App<'a> {
    is_coord: bool,
    state: Option<Box<dyn StateImpl>>,
    cmd_tx: &'a flume::Sender<usi::Message>,
}

impl<'a> App<'a> {
    pub fn new(is_coord: bool, cmd_tx: &'a flume::Sender<usi::Message>) -> Self {
        App {
            is_coord,
            state: None,
            cmd_tx: cmd_tx,
        }
    }

    pub fn process_msg(&mut self, msg: &usi::Message) -> bool {
        log::info!("App processing message {:?}", msg);
        if let Some(s) = &self.state {
            match msg {
                usi::Message::UsiIn(msg) => {
                    if let Some(m) = adp::usi_message_to_message(msg) {
                        let new_state = s.on_msg(self, &m);
                        if new_state.is_none() {
                            log::info!("State {}, on exit", s.get_name());
                            s.on_exit(self);
                            return false;
                        }
                        let new_state = new_state.unwrap();
                        if new_state.get_name() != s.get_name() {
                            log::info!("State {}, on exit", s.get_name());
                            s.on_exit(self);

                            log::info!("State {}, on enter", new_state.get_name());
                            self.state = Some(new_state);
                            if let Some(s) = &self.state {
                                s.on_enter(self);
                            }
                        }
                    } else {
                        log::warn!("Adp failed to process message ...");
                    }
                }
                usi::Message::UsiOut(_) => {
                    log::warn!("App received a UsiCommand");
                }
                usi::Message::HeartBeat(time) => {
                    log::info!("Adp received heartbeat {:?}", time);
                }
                _ => {}
            }
        } else {
            match msg {
                usi::Message::SystemStartup => {
                    self.state = Some(Box::new(Start {}));
                    if let Some(s) = &self.state {
                        s.on_enter(self);
                    }
                }
                _ => {}
            }
        }
        return true;
    }
}

#[derive(PartialEq, Eq)]
struct Start {}
impl StateImpl for Start {
    fn on_enter(&self, app: &App) {
        let cmd = request::AdpInitializeRequest::from_band(adp::TAdpBand::ADP_BAND_CENELEC_A); //TODO parameterize
        if let Ok(c) = cmd.try_into() {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("received message : {:?}", msg);
        return Some(Box::new(SetupParameters::new()));
    }

    fn on_exit(&self, app: &App) {}

    fn get_name(&self) -> &str {
        "Start"
    }
}


#[derive(Clone)]
struct SetupParameters {
    parameters: Rc<RefCell<Vec<common::Parameter>>>,
}
impl SetupParameters {
    pub fn new() -> SetupParameters {
        let parameters = Rc::new(RefCell::new(Vec::with_capacity(10)));

        // parameters.borrow_mut().push(common::Parameter::new(common::PROTOCOL_ADP_G3, adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(), 0, [0x0].to_vec()));

        // let mut v = vec![0x31, 0x30, 0x36, 0x33, 0x4C, 0x50, 0x54, 0x41];
        // v.reverse();

        // parameters.borrow_mut().push(common::Parameter::new (common::PROTOCOL_MAC_G3, adp::EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS.into(),
        //             0, v));

        parameters.borrow_mut().push(common::Parameter::new(
            common::PROTOCOL_MAC_G3,
            adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS.into(),
            0,
            vec![0x0, 0x0],
        ));

        // let mut v = vec![0x02,0x00,0x01,0x50,0xFE,0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x78,0x1D];
        // v.reverse();
        // parameters.borrow_mut().push(common::Parameter::new(common::PROTOCOL_ADP_G3,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 0, v));
        // let mut v = vec![0x02,0x00,0x01,0x30,0x11,0x22,0x33,0x44,0x55,0x66];
        // v.reverse();
        // parameters.borrow_mut().push(common::Parameter::new(common::PROTOCOL_ADP_G3, adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 1, v));
        parameters.borrow_mut().push(common::Parameter::new(
            common::PROTOCOL_ADP_G3,
            adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(),
            0,
            vec![0x00, 0xB4],
        ));
        parameters.borrow_mut().push(common::Parameter::new(
            common::PROTOCOL_ADP_G3,
            adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME.into(),
            0,
            vec![0x00, 0x5A],
        ));
        parameters.borrow_mut().push(common::Parameter::new(
            common::PROTOCOL_ADP_G3,
            adp::EAdpPibAttribute::ADP_IB_MAX_HOPS.into(),
            0,
            vec![0x0A],
        ));
        // let mut val = vec![0xAB,0x10,0x34,0x11,0x45,0x11,0x1B,0xC3,0xC1,0x2D,0xE8,0xFF,0x11,0x14,0x22,0x04];
        // val.reverse();
        // parameters.borrow_mut().push(common::Parameter::new(common::PROTOCOL_ADP_G3, adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY.into(), 0, val));

        SetupParameters { parameters }
    }

    fn send_parameter(&self, app: &App, p: common::Parameter) {
        match p.protocol {
            common::PROTOCOL_ADP_G3 => {
                if let Ok(attribute) = adp::EAdpPibAttribute::try_from_primitive(p.id) {
                    if let Ok(c) = request::AdpSetRequest::new(attribute, p.idx, &p.value).try_into()
                    {
                        app.cmd_tx.send(usi::Message::UsiOut(c));
                    }
                }
            }
            common::PROTOCOL_MAC_G3 => {
                if let Ok(attribute) = adp::EMacWrpPibAttribute::try_from_primitive(p.id) {
                    if let Ok(c) =
                        request::AdpMacSetRequest::new(attribute, p.idx, &p.value).try_into()
                    {
                        app.cmd_tx.send(usi::Message::UsiOut(c));
                    }
                }
            }
            _ => {}
        }
    }
}
impl StateImpl for SetupParameters {
    fn on_enter(&self, app: &App) {
        self.parameters.borrow_mut().reverse();
        if let Some(p) = self.parameters.borrow_mut().pop() {
            self.send_parameter(app, p);
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("received message : {:?}", msg);
        if let Some(p) = self.parameters.borrow_mut().pop() {
            self.send_parameter(app, p);
            let parameters = self.parameters.clone();
            return Some(Box::new(SetupParameters {
                parameters: parameters,
            }));
        } else if app.is_coord {
            log::info!("starting coordinator");
            return Some(Box::new(NetworkStart {}));
        } else {
            log::info!("not coordinator");
            return Some(Box::new(Idle {}));
        }
    }

    fn on_exit(&self, app: &App) {}

    fn get_name(&self) -> &str {
        "SetupParameters"
    }
}
#[derive(PartialEq, Eq)]
struct Idle {}
impl StateImpl for Idle {
    fn on_enter(&self, app: &App) {
        if !app.is_coord {
            let discovery = request::AdpDiscoveryRequest::new(SCAN_DURATION);
            if let Ok(c) = discovery.try_into() {
                app.cmd_tx.send(usi::Message::UsiOut(c));
            }
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("received message : {:?}", msg);
        // return Some(Box::new(GetVersion{}));
        match msg {
            adp::Message::AdpG3DiscoveryResponse(msg) => {
                if !app.is_coord {}
                let discovery = request::AdpDiscoveryRequest::new(SCAN_DURATION);
                if let Ok(c) = discovery.try_into() {
                    app.cmd_tx.send(usi::Message::UsiOut(c));
                }
            }
            _ => {}
        }
        return Some(Box::new(Idle {}));
    }

    fn on_exit(&self, app: &App) {}

    fn get_name(&self) -> &str {
        "Idle"
    }
}

struct SetSecurityLevel {}
impl StateImpl for SetSecurityLevel {
    fn get_name(&self) -> &str {
        "SetSecurityLevel"
    }

    fn on_enter(&self, app: &App) {
        let v:Vec<u8> = vec![0; 16];
        let c = request::AdpSetRequest::new(
            adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY,
            0,
            &v
        );
        if let Ok(c) = c.try_into() {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("state {}, msg {:?}", self.get_name(), msg);
        return Some(Box::new(GetEUI64 {}));
    }

    fn on_exit(&self, app: &App) {}
}
struct GetVersion {}

impl StateImpl for GetVersion {
    fn get_name(&self) -> &str {
        "GetVersion"
    }

    fn on_enter(&self, app: &App) {
        let get_version =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_SOFT_VERSION, 0);
        if let Ok(c) = get_version.try_into() {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("state {}, msg {:?}", self.get_name(), msg);
        return Some(Box::new(SetSecurityLevel {}));
    }

    fn on_exit(&self, app: &App) {}
}

struct GetEUI64 {}

impl StateImpl for GetEUI64 {
    fn get_name(&self) -> &str {
        "GetEUI64"
    }

    fn on_enter(&self, app: &App) {
        let get_eui64 = request::AdpMacGetRequest::new(
            adp::EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS,
            0,
        );
        if let Ok(c) = get_eui64.try_into() {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpMacGetRequest::new(adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS, 0)
                .try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpMacGetRequest::new(adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS, 0)
                .try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        // if let Ok(c) =
        //     request::AdpMacGetRequest::new(adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID, 0)
        //         .try_into()
        // {
        //     app.cmd_tx.send(usi::Message::UsiOut(c));
        // }
        if let Ok(c) =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 0)
                .try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE, 1)
                .try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL, 0)
                .try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME, 0)
                .try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_MAX_HOPS, 0).try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY, 0)
                .try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
        if let Ok(c) =
            request::AdpGetRequest::new(adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL, 0).try_into()
        {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("state {}, msg {:?}", self.get_name(), msg);
        return Some(Box::new(GetEUI64 {}));
    }

    fn on_exit(&self, app: &App) {}
}

struct NetworkStart {}

impl StateImpl for NetworkStart {
    fn get_name(&self) -> &str {
        "NetworkStart"
    }

    fn on_enter(&self, app: &App) {
        let network_start = request::AdpNetworkStartRequest::new(PAN_ID);
        if let Ok(c) = network_start.try_into() {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("state {}, msg {:?}", self.get_name(), msg);
        Some(Box::new(Idle {}))
    }

    fn on_exit(&self, app: &App) {}
}

struct GetAttributes {
    extended_address: Option<adp::TAddress>,
}

impl StateImpl for GetAttributes {
    fn get_name(&self) -> &str {
        "GetParams"
    }

    fn on_enter(&self, app: &App) {
        let get_eui64 = request::AdpMacGetRequest::new(
            adp::EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS,
            0,
        );
        if let Ok(c) = get_eui64.try_into() {
            app.cmd_tx.send(usi::Message::UsiOut(c));
        }
    }

    fn on_msg(&self, app: &App, msg: &adp::Message) -> Option<Box<dyn StateImpl>> {
        log::info!("state {}, msg {:?}", self.get_name(), msg);
        match msg {
            adp::Message::AdpG3GetResponse(get_response) => {}
            adp::Message::AdpG3GetMacResponse(get_mac_response) => {}
            _ => {}
        }
        None
    }

    fn on_exit(&self, app: &App) {}
}

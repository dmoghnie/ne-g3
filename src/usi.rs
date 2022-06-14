use log::{trace, warn};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    thread,
    time::{Duration, SystemTime},
};

use crate::common::{self, array_to_hex_string, to_hex_string, PROTOCOL_PRIME_API};
use crate::crc;
use crate::usi;

// use crossbeam_channel::{bounded, Sender};

//use std::sync::mpsc::{Sender, channel};

//TODO, this is a half duplex implementation, one thread for rx/tx.
//When receiving, we timeout in x time and check the incoming channel for pending requests

#[derive(Debug)]
pub enum Message {
    UsiIn(InMessage),
    UsiOut(OutMessage),
    HeartBeat(SystemTime),
    SystemStartup,
}

pub trait MessageHandler {
    fn process(&mut self, msg: usi::Message) -> bool;
}

#[derive(Clone, Debug)]
enum RxState {
    RxIdle, // Inactive
    RxMsg,  // Receiving message
    RxEsc,  // Processing escape char
    RxError,
    RxDone,
}

pub const RECEIVE_TIMEOUT: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct OutMessage {
    protocol: u8,
    data: Vec<u8>,
}

impl OutMessage {
    pub fn new(protocol: u8, data: &Vec<u8>) -> OutMessage {
        OutMessage {
            protocol: protocol,
            data: data.to_vec(),
        }
    }
    pub fn to_usi(&self) -> Option<Vec<u8>> {
        let mut v: Vec<u8> = Vec::with_capacity(1024); //TODO define those limits
                                                       //Header is 2 bytes
                                                       //Cmd is first byte in the data
                                                       //in case of prime, the cmd byte when serializing contains the ex len

        if self.data.len() > 2048 {
            //TODO set const
            return None;
        }
        v.push(common::LEN_HI_PROTOCOL(self.data.len() as u16));
        v.push(
            common::LEN_LO_PROTOCOL(self.data.len() as u16) + common::TYPE_PROTOCOL(self.protocol),
        );

        if let Some(cmd) = self.data.get(0) {
            if self.protocol == common::PROTOCOL_PRIME_API {
                v.push(
                    common::LEN_EX_PROTOCOL(self.data.len() as u16) + common::CMD_PROTOCOL(*cmd),
                );
            } else {
                v.push(*cmd);
            }
        }
        for ch in self.data.iter().skip(1) {
            v.push(*ch);
        }
        // Calculate CRC
        match self.protocol {
            common::MNGP_PRIME_GETQRY
            | common::MNGP_PRIME_GETRSP
            | common::MNGP_PRIME_SET
            | common::MNGP_PRIME_RESET
            | common::MNGP_PRIME_REBOOT
            | common::MNGP_PRIME_FU
            | common::MNGP_PRIME_EN_PIBQRY => {
                let crc = crc::evalCrc32(&v);
                v.push((crc >> 24) as u8);
                v.push((crc >> 16) as u8);
                v.push((crc >> 8) as u8);
                v.push(crc as u8);
            }

            common::PROTOCOL_SNIF_PRIME
            | common::PROTOCOL_SNIF_G3
            | common::PROTOCOL_MAC_G3
            | common::PROTOCOL_ADP_G3
            | common::PROTOCOL_COORD_G3 => {
                let crc = crc::evalCrc16(&v);
                v.push((crc >> 8) as u8);
                v.push(crc as u8);
            }

            common::PROTOCOL_PRIME_API => {
                let crc = crc::evalCrc8(&v);
                v.push(crc as u8);
            }
            common::PROTOCOL_PRIMEoUDP => {
                let crc = crc::evalCrc16(&v);
                v.push((crc >> 8) as u8);
                v.push(crc as u8);
            }

            _ => {
                let crc = crc::evalCrc8(&v);
                v.push(crc as u8);
            }
        }
        let mut r: Vec<u8> = Vec::with_capacity(2048);
        for ch in v {
            if ch == common::MSGMARK || ch == common::ESCMARK {
                r.push(common::ESCMARK);
                r.push(ch ^ 0x20);
            } else {
                r.push(ch);
            }
        }
        r.insert(0, common::MSGMARK);
        r.push(common::MSGMARK);
        return Some(r);
    }
}

pub trait UsiSender {
    fn send(&mut self, cmd: &OutMessage) -> std::result::Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct InMessage {
    pub buf: Vec<u8>,
    rxState: RxState,
    pub protocol_type: Option<u8>,
    payload_len: usize,
}

impl InMessage {
    pub fn new() -> Self {
        InMessage {
            rxState: RxState::RxIdle,
            buf: Vec::with_capacity(1024),
            payload_len: 0,
            protocol_type: None,
        }
    }
    pub fn get_state(&self) -> &RxState {
        return &self.rxState;
    }
    fn remove_header_and_crc(&mut self) {
        if self.buf.len() >= 2 {
            self.buf = self.buf[2..].to_vec();
        }
        self.buf = self.buf[..self.payload_len].to_vec();
        if self.protocol_type == Some(PROTOCOL_PRIME_API) {
            self.buf[0] = common::CMD_PROTOCOL(self.buf[0]);
        }
    }
    fn process_header(&mut self) {
        if self.buf.len() < common::PROTOCOL_MIN_LEN.into() {
            return;
        }
        if let Some(proto) = self.buf.get(common::TYPE_PROTOCOL_OFFSET as usize) {
            self.protocol_type = common::TYPE_PROTOCOL(*proto).into();
            if self.protocol_type.unwrap() == common::PROTOCOL_PRIME_API {
                if let (Some(b1), Some(b2), Some(b3)) = (
                    self.buf.get(common::LEN_PROTOCOL_HI_OFFSET as usize),
                    self.buf.get(common::LEN_PROTOCOL_LO_OFFSET as usize),
                    self.buf.get(common::XLEN_PROTOCOL_OFFSET as usize),
                ) {
                    self.payload_len = common::get_protocol_xlen(*b1, *b2, *b3).into();
                }
            } else {
                if let (Some(b1), Some(b2)) = (
                    self.buf.get(common::LEN_PROTOCOL_HI_OFFSET as usize),
                    self.buf.get(common::LEN_PROTOCOL_LO_OFFSET as usize),
                ) {
                    self.payload_len = common::get_protocol_len(*b1, *b2).into();
                }
            }
        }
    }
    fn check_crc(&self) -> bool {
        if let Some(pt) = self.protocol_type {
            match pt {
                common::MNGP_PRIME_GETQRY
                | common::MNGP_PRIME_GETRSP
                | common::MNGP_PRIME_SET
                | common::MNGP_PRIME_RESET
                | common::MNGP_PRIME_REBOOT
                | common::MNGP_PRIME_FU
                | common::MNGP_PRIME_EN_PIBQRY
                | common::MNGP_PRIME_EN_PIBRSP => {
                    let crc_len = 4;
                    if let Some(tb) = self.buf.get(self.buf.len() - (crc_len)..) {
                        let rxCrc: u32 = (tb[0] as u32) << 24
                            | (tb[1] as u32) << 16
                            | (tb[2] as u32) << 8
                            | tb[3] as u32;
                        if let Some(d) = self.buf.get(0..(self.payload_len + 2)) {
                            return rxCrc == crc::evalCrc32(&d.to_vec());
                        }
                    }
                }
                common::PROTOCOL_SNIF_PRIME
                | common::PROTOCOL_SNIF_G3
                | common::PROTOCOL_MAC_G3
                | common::PROTOCOL_ADP_G3
                | common::PROTOCOL_COORD_G3
                | common::PROTOCOL_PRIMEoUDP => {
                    let crc_len = 2;
                    if let Some(tb) = self.buf.get(self.buf.len() - (crc_len)..) {
                        let rxCrc = (tb[0] as u16) << 8 | (tb[1] as u16);
                        if let Some(d) = self.buf.get(0..(self.payload_len + 2)) {
                            return rxCrc == crc::evalCrc16(&d.to_vec());
                        }
                    }
                }
                common::PROTOCOL_PRIME_API => {
                    let crc_len = 1;
                    if let Some(tb) = self.buf.get(self.buf.len() - (crc_len)..) {
                        let rxCrc = tb[0];
                        if let Some(d) = self.buf.get(0..(self.payload_len as usize + 2)) {
                            return rxCrc == crc::evalCrc8(&d.to_vec());
                        }
                    }
                }
                _ => {}
            }
        }
        return false;
    }
    pub fn process(&mut self, data: &mut VecDeque<u8>) {
        loop {
            if let Some(c) = data.pop_front() {
                self.process_ch(c);
                match (self.rxState) {
                    RxState::RxDone => {
                        break;
                    }
                    RxState::RxError => {
                        break;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
    }

    fn process_ch(&mut self, ch: u8) {
        // trace!("usi::process_ch {}", ch);
        match self.rxState {
            RxState::RxIdle => {
                if ch == common::PROTOCOL_DELIMITER {
                    self.rxState = RxState::RxMsg;
                }
            }
            RxState::RxMsg => {
                if ch == common::PROTOCOL_ESC {
                    self.rxState = RxState::RxEsc;
                } else if ch == common::PROTOCOL_DELIMITER {
                    if self.buf.is_empty() {
                        //Two consecutive 0x7e received
                        //The first was ending of a non processed message
                        //the second is the begining of next message to process
                    } else {
                        if self.check_crc() {
                            self.rxState = RxState::RxDone;
                        } else {
                            println!("CRC failed"); //TODO return Result
                            self.rxState = RxState::RxError;
                        }
                    }
                } else {
                    self.buf.push(ch);
                    if self.protocol_type == None {
                        self.process_header();
                    }
                }
            }
            RxState::RxEsc => {
                if ch == common::PROTOCOL_ESC {
                    println!("Received ESC in Msg state");
                    self.rxState = RxState::RxError;
                } else {
                    self.buf.push(ch ^ 0x20);
                    self.process_header();
                    self.rxState = RxState::RxMsg;
                }
            }
            _ => {}
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum PortState {
    Stopped,
    Starting,
    Started,
    Stopping,
}

pub struct Port<'a, T> {
    message: usi::InMessage,
    buf: VecDeque<u8>,
    channel: T,
    state: &'a PortState,
    listeners: Vec<flume::Sender<Message>>,
}

//thread object should be static, makes sense! Since threads may live for the duration of the program
impl<'a, T: Read + Write + Send> Port<'a, T>
where
    'a: 'static,
    T: 'static,
{
    pub fn new(channel: T) -> Port<'a, T> {
        Port {
            message: usi::InMessage::new(),
            buf: VecDeque::with_capacity(2048),
            channel: channel,
            state: &PortState::Stopped,
            listeners: Vec::new(),
        }
    }
    // pub fn post_cmd(&self, cmd: &'a MessageType) {
    //     self.cmd_tx_rx.0.send(cmd);
    // }
    pub fn add_listener(&mut self, listener: flume::Sender<Message>) {
        self.listeners.push(listener);
    }

    // pub fn process<T>(&mut self, port: &mut T, listener:&Box<dyn message::MessageListener>) -> Option<Vec<u8>>
    fn process(&mut self) {
        let mut b = [0; 2048];

        match self.channel.read(&mut b) {
            Ok(t) => {
                if t == 0 {
                    return;
                } else {
                    trace!("usi received {} ", array_to_hex_string(b[..t].to_vec()));
                }
                for ch in &mut b[..t] {
                    //TODO, push whole slice
                    self.buf.push_back(*ch);
                }
                // self.buf.append(b[..t].to_vec());
                // let ch = b[0];
                // trace!("Received {}", ch);
                self.message.process(&mut self.buf);
                match self.message.get_state() {
                    usi::RxState::RxDone => {
                        self.message.remove_header_and_crc();
                        for listener in &self.listeners {
                            listener.send(usi::Message::UsiIn(self.message.clone()));
                        }
                        // if let Some(ref sender) = sender {
                        //     sender.send(self.message.clone());
                        // }
                        self.message = usi::InMessage::new();
                    }
                    usi::RxState::RxError => {
                        log::warn!("Failed to parse message : RxError");
                        self.message = usi::InMessage::new();
                    }
                    _ => {}
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => {
                warn!("Error {}", e);
            }
        }
    }
    pub fn start(mut self) -> flume::Sender<Message> {
        let (tx, rx) = flume::unbounded::<Message>();
        thread::spawn(move || loop {
            self.process();
            match rx.recv_timeout(usi::RECEIVE_TIMEOUT) {
                Ok(msg) => match msg {
                    Message::UsiOut(cmd) => {
                        self.send(&cmd);
                    }
                    _ => {}
                },
                Err(e) => {}
            }
        });
        tx
    }
}

impl<'a, T: Read + Write + Send> UsiSender for Port<'a, T> {
    fn send(&mut self, cmd: &OutMessage) -> std::result::Result<(), String> {
        if let Some(buf) = cmd.to_usi() {
            log::trace!("--> {}", common::to_hex_string(&buf));
            match self.channel.write_all(&buf) {
                Ok(()) => return Ok(()),
                Err(ref e) => return Err(String::from(e.to_string())),
            }
        }
        return Err(String::from("Invalid UsiCommand"));
    }
}

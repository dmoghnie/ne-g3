use bytes::{BufMut, BytesMut};
use log::{trace, warn};
use std::{
    borrow::BorrowMut,
    collections::VecDeque,
    io::{Read, Write},
    ops::Index,
    os::unix::thread,
};

use crate::common;
use crate::crc;
use crate::usi;

use crate::message;
use crossbeam_channel::{bounded, Sender};
use std::thread::spawn;

#[derive(Clone)]
enum RxState {
    RxIdle, // Inactive
    RxMsg,  // Receiving message
    RxEsc,  // Processing escape char
    RxError,
    RxDone,
}

pub struct UsiCommand {
    protocol: u8,    
    data: Vec<u8>
}

impl UsiCommand {
    pub fn new(protocol: u8, data: &Vec<u8>) -> UsiCommand{
        UsiCommand {
            protocol: protocol,
            data: data.to_vec()
        }
    }
    pub fn to_usi (&self)->Option<Vec<u8>> {
        let mut v:Vec<u8> = Vec::with_capacity(1024); //TODO define those limits
        //Header is 2 bytes
        //Cmd is first byte in the data
        //in case of prime, the cmd byte when serializing contains the ex len

        if self.data.len() > 2048 { //TODO set const
            return None;
        } 
        v.push(common::LEN_HI_PROTOCOL(self.data.len() as u16));
        v.push(common::LEN_LO_PROTOCOL(self.data.len() as u16) + common::TYPE_PROTOCOL(self.protocol));
        
        
        if let Some(cmd) = self.data.get(0) {
            if self.protocol == common::PROTOCOL_PRIME_API {
                v.push(common::LEN_EX_PROTOCOL(self.data.len() as u16) + common::CMD_PROTOCOL(*cmd));
            }            
            else {
                v.push(*cmd);
            }
        }
        for ch in self.data.iter().skip(1) {
            v.push(*ch);
        }
        // Calculate CRC
	match self.protocol
	{
		common::MNGP_PRIME_GETQRY |
		common:: MNGP_PRIME_GETRSP |
		common::MNGP_PRIME_SET |
		common::MNGP_PRIME_RESET |
		common::MNGP_PRIME_REBOOT |
		common::MNGP_PRIME_FU |
        common::MNGP_PRIME_EN_PIBQRY => {
			let crc = crc::evalCrc32 (&v);
            v.push((crc >> 24) as u8);
            v.push((crc >> 16) as u8);
            v.push((crc >> 8) as u8);
            v.push(crc as u8);
        },

        common::PROTOCOL_SNIF_PRIME|
        common::PROTOCOL_SNIF_G3|
        common::PROTOCOL_MAC_G3|
        common::PROTOCOL_ADP_G3|
        common::PROTOCOL_COORD_G3 => {
            let crc = crc::evalCrc16 (&v);
            v.push((crc >> 8) as u8);
            v.push(crc as u8);
        },

        common::PROTOCOL_PRIME_API=> {
            let crc = crc::evalCrc8(&v);
            v.push(crc as u8);

        },
        common::PROTOCOL_PRIMEoUDP => {
            let crc = crc::evalCrc16 (&v);
            v.push((crc >> 8) as u8);
            v.push(crc as u8);
        },

        _ => {
            let crc = crc::evalCrc8(&v);
            v.push(crc as u8);
        }
	}
        let mut r:Vec<u8> = Vec::with_capacity(2048);
        for ch in v{ 
            if ch == common::MSGMARK || ch == common::ESCMARK {
                r.push(common::ESCMARK);
                r.push (ch ^ 0x20);
            }
            else {
                r.push(ch);
            }
        }
        r.insert(0, common::MSGMARK);
        r.push(common::MSGMARK);
        return Some(r);
    }
}

pub trait UsiSender {
    fn send (&mut self, cmd: &UsiCommand) -> std::result::Result<usize, String>;
}

#[derive(Clone)]
pub struct UsiMessage {
    pub buf: Vec<u8>,
    rxState: RxState,
    pub protocol_type: Option<u8>,
    payload_len: Option<u16>,
}

impl UsiMessage {
    pub fn new() -> Self {
        UsiMessage {
            rxState: RxState::RxIdle,
            buf: Vec::with_capacity(1024),
            payload_len: None,
            protocol_type: None,
        }
    }
    pub fn get_state(&self) -> &RxState {
        return &self.rxState;
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
        if let (Some(pt), Some(pl)) = (self.protocol_type, self.payload_len) {
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
                        if let Some(d) = self.buf.get(0..(pl as usize + 2)) {
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
                        if let Some(d) = self.buf.get(0..(pl as usize + 2)) {
                            return rxCrc == crc::evalCrc16(&d.to_vec());
                        }
                    }
                }
                common::PROTOCOL_PRIME_API => {
                    let crc_len = 1;
                    if let Some(tb) = self.buf.get(self.buf.len() - (crc_len)..) {
                        let rxCrc = tb[0];
                        if let Some(d) = self.buf.get(0..(pl as usize + 2)) {
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
    pub fn get_message(&self) -> Option<&[u8]> {
        if let Some(len) = self.payload_len {
            return self.buf.get(2..(len as usize + 2));
        }
        None
    }
    fn process_ch(&mut self, ch: u8) {
        trace!("usi::process_ch {}", ch);
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
                    if self.protocol_type == None || self.payload_len == None {
                        self.process_header();
                    }
                }
            }
            RxState::RxEsc => {
                if ch == common::PROTOCOL_ESC {
                    println!("Received ESC in Msg state");
                    self.rxState = RxState::RxError;
                } else {
                    self.buf.push(ch ^ 20);
                    self.process_header();
                    self.rxState = RxState::RxMsg;
                }
            }
            _ => {}
        }
    }
}

pub struct Port<T> {
    message: usi::UsiMessage,
    buf: VecDeque<u8>,
    channel: T,
}

impl<T: Read + Write + Send> Port<T> {
    pub fn new(channel: T) -> Port<T> {
        Port {
            message: usi::UsiMessage::new(),
            buf: VecDeque::with_capacity(2048),
            channel: channel,
        }
    }
    // pub fn process<T>(&mut self, port: &mut T, listener:&Box<dyn message::MessageListener>) -> Option<Vec<u8>>
    pub fn process(&mut self, sender: Option<Sender<UsiMessage>>) {
        loop {
            let mut b = [0; 1000];

            match self.channel.read(&mut b) {
                Ok(t) => {
                    if t == 0 {
                        continue;
                    } else {
                        trace!("usi received {} bytes", t);
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
                            if let Some(ref sender) = sender {
                                sender.send(self.message.clone());
                            }
                            self.message = usi::UsiMessage::new();
                        }
                        usi::RxState::RxError => {
                            log::warn!("Failed to parse message : RxError");
                            self.message = usi::UsiMessage::new();
                        }
                        _ => {}
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
                Err(e) => {
                    warn!("Error {}", e);
                    break;
                }
            }
        }
    }
}

impl<T: Read + Write + Send> UsiSender for Port<T> {
    fn send(&mut self, cmd: &UsiCommand) -> std::result::Result<usize, String> {
        if let Some(buf) = cmd.to_usi() {
            log::trace!("--> {}", common::to_hex_string(&buf));
            match self.channel.write(&buf) {
                Ok(len) => return Ok(len),
                Err(ref e) => return Err(String::from(e.to_string())),
            }
        }
        return Err(String::from("Invalid UsiCommand"));
    }
}

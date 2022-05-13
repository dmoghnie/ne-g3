
mod usi;
mod message;
mod common;
mod crc;
mod request;

use std::{env, io, str, thread};
use std::time::Duration;

use bytes::BytesMut;

use std::io::{Read, Result as IoResult, Write};
use crossbeam_channel::{bounded, Sender};


#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/tty.usbserial-0001";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

#[macro_use] extern crate log;
extern crate env_logger;

use log::Level;

use crate::usi::UsiSender;




fn main() {
    let (adp_msg_sender, adp_msg_receiver) = bounded(100);

    env_logger::init();
    info!("Starting ...");
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());

    let mut port = serialport::new(DEFAULT_TTY, 230_400)
    .timeout(Duration::from_millis(10))
    .open().expect("Failed to open port");

    let mut usi_port = usi::Port::new(Box::new(port));    

    let request = request::AdpInitializeRequest::from_band(message::TAdpBand::ADP_BAND_CENELEC_A);
    match usi_port.send (&request.try_into().unwrap()){
        Ok(size) => {
            trace!("sent cmd result ok {}", size)
        },
        Err(e) => {
            warn!("Failed to send cmd {}", e)
        }
    };
    // let listener = Box::new(Listener()) as Box<dyn MessageListener>;
    let handle = thread::spawn (move || {
        loop {
            // device.process(&mut port, &listener);
            usi_port.process(Some(adp_msg_sender.clone()));
    
        }
    });

    let receiver = thread::spawn(move || {
        loop {
            let msg = adp_msg_receiver.recv();
            match msg {
                Ok(msg) => {
                    let m = message::usi_message_to_message (&msg);
                    log::trace!("Receiver received {:?}", m);
                },
                Err(e) => {
                    log::warn!("Error receiving message");
                }
            }
        }
    });
    
    handle.join().unwrap();

}

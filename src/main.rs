
mod usi;
mod message;
mod common;
mod crc;
mod request;
mod app;

use std::{env, io, str, thread};
use std::time::{Duration, SystemTime};

use bytes::BytesMut;
use flume::{Sender, Receiver};

use std::io::{Read, Result as IoResult, Write};
// use crossbeam_channel::{bounded, Sender};

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/tty.usbserial-0001";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

#[macro_use] extern crate log;
extern crate env_logger;

use log::Level;

use crate::usi::{UsiSender, UsiCommand, MessageType};


const TIMER_RESOLUTION:Duration = Duration::from_millis(10000);

fn main() {
    // let (tx, rx) = flume::unbounded();
    // let (adp_msg_sender, adp_msg_receiver) = bounded(100);

    env_logger::init();
    info!("Starting ...");
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());
    log::trace!("Port : {}", &tty_path);
    let mut port = serialport::new(tty_path, 230_400)
    .timeout(Duration::from_millis(10))
    .open().expect("Failed to open port {}");

    let (app_tx, app_rx) = flume::unbounded::<(MessageType)>();
    let (usi_tx, usi_rx) = flume::unbounded::<MessageType>();
        
    // let request = request::AdpInitializeRequest::from_band(message::TAdpBand::ADP_BAND_CENELEC_A);
    let sender = app_tx.clone();
    let t = thread::spawn(move || {
        let mut usi_port = usi::Port::new(Box::new(port));
        // log::trace!("Listening on port {}", tty_path);
        usi_port.add_listener (sender);
        loop {
            usi_port.process();
            match usi_rx.recv_timeout(usi::RECEIVE_TIMEOUT) {
                Ok(msg) => {
                    match msg {
                        MessageType::UsiCommand(cmd) => {
                            usi_port.send(&cmd);
                        },
                        _=> {

                        }
                    }
                },
                Err(e) =>{

                }
            }
        }

    });

    let cmd_tx = usi_tx.clone();
    
    let t2 = thread::spawn(move || {
        let mut app = app::App::new(&cmd_tx);    
        loop {
            match app_rx.recv() {
                Ok(msg) => {
                    if !app.process_msg (&msg) {
                        break;
                    }
                },
                Err(e) => {

                }
            }
        }
    });

    let system_tx = app_tx.clone();
    system_tx.send (MessageType::SystemStartup);
    let system_handle = thread::spawn(move || {
        loop {
            system_tx.send(MessageType::HeartBeat(SystemTime::now()));
            thread::sleep(TIMER_RESOLUTION);
        }
    });
    t2.join().unwrap();
    // match usi_port.send (&request.try_into().unwrap()){
    //     Ok(size) => {
    //         trace!("sent cmd");
    //     },
    //     Err(e) => {
    //         warn!("Failed to send cmd {}", e);
    //     }
    // };
    // // let listener = Box::new(Listener()) as Box<dyn MessageListener>;
    // let handle = thread::spawn (move || {
    //     loop {
    //         // device.process(&mut port, &listener);
    //         usi_port.process(Some(&tx));
    
    //     }
    // });

    // let receiver = thread::spawn(move || {
    //     loop {
    //         let msg = rx.recv();
    //         match msg {
    //             Ok(msg) => {
    //                 let m = message::usi_message_to_message (&msg);
    //                 log::trace!("Receiver received {:?}", m);
    //             },
    //             Err(e) => {
    //                 log::warn!("Error receiving message");
    //             }
    //         }
    //     }
    // });
    
    // handle.join().unwrap();

}

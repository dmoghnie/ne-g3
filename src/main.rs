mod adp;
mod app;
mod common;
mod coord;
mod crc;
mod lbp;
mod modem;
mod request;
mod usi;

use std::time::{Duration, SystemTime};
use std::{env, io, str, thread};

use bytes::BytesMut;
use env_logger::Env;
use flume::{Receiver, Sender};

use std::io::{Read, Result as IoResult, Write};
// use crossbeam_channel::{bounded, Sender};

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/tty.usbserial-0001";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

#[macro_use]
extern crate log;
extern crate env_logger;

use log::Level;

use crate::usi::{Message, MessageHandler, OutMessage, UsiSender};

const TIMER_RESOLUTION: Duration = Duration::from_millis(20000);

fn main() {
    // let (tx, rx) = flume::unbounded();
    // let (adp_msg_sender, adp_msg_receiver) = bounded(100);

    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting ...");
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());
    let is_coordinator = args
        .nth(0)
        .unwrap_or("true".into())
        .parse()
        .unwrap_or(false);

    log::trace!("Port : {}, coordinator {}", &tty_path, is_coordinator);
    let mut port = serialport::new(tty_path, 230_400)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port {}");

    let (app_tx, app_rx) = flume::unbounded::<(Message)>();
    let (usi_tx, usi_rx) = flume::unbounded::<Message>();

    // let request = request::AdpInitializeRequest::from_band(message::TAdpBand::ADP_BAND_CENELEC_A);
    let sender = app_tx.clone();
    let t = thread::spawn(move || {
        let mut usi_port = usi::Port::new(Box::new(port));
        // log::trace!("Listening on port {}", tty_path);
        usi_port.add_listener(sender);
        loop {
            usi_port.process();
            match usi_rx.recv_timeout(usi::RECEIVE_TIMEOUT) {
                Ok(msg) => match msg {
                    Message::UsiOut(cmd) => {
                        usi_port.send(&cmd);
                    }
                    _ => {}
                },
                Err(e) => {}
            }
        }
    });

    let cmd_tx = usi_tx.clone();

    // let t2 = thread::spawn(move || {
    //     let mut app = app::App::new(is_coordinator ,&cmd_tx);
    //     loop {
    //         match app_rx.recv() {
    //             Ok(msg) => {
    //                 if !app.process_msg(&msg) {
    //                     break;
    //                 }
    //             }
    //             Err(e) => {}
    //         }
    //     }
    // });

    let t2 = thread::spawn(move || {
        let mut message_handler: Option<Box<dyn MessageHandler>>;
        if is_coordinator {
            message_handler = Some(Box::new(coord::Coordinator::new(cmd_tx)));
            
        } else {
            message_handler = Some(Box::new(modem::Modem::new(cmd_tx)));
        }
        if let Some(mut handler) = message_handler {
            loop {
                match app_rx.recv() {
                    Ok(msg) => {
                        if !handler.process(msg) {
                            break;
                        }
                    }
                    Err(e) => {}
                }
            }
        }
    });

    let system_tx = app_tx.clone();
    system_tx.send(Message::SystemStartup);
    let system_handle = thread::spawn(move || loop {
        system_tx.send(Message::HeartBeat(SystemTime::now()));
        thread::sleep(TIMER_RESOLUTION);
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

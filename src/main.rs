mod adp;
mod app;
mod app_config;
mod common;
mod coord;
mod crc;
mod lbp;
mod lbp_functions;
mod lbp_manager;
mod modem;
mod network_manager;
mod request;
mod usi;

use std::time::{Duration, SystemTime};
use std::{env, io, str, thread};

use bytes::BytesMut;
use env_logger::Env;
use flume::{Receiver, Sender};
use packet::ip::v6::Packet;

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
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    let s = app_config::SETTINGS.read().unwrap();

    log::trace!("Settings : {:?}", s);

    log::trace!(
        "CONF_CONTEXT_INFORMATION_TABLE_0 : {:X?}",
        network_manager::NetworkManager::CONF_CONTEXT_INFORMATION_TABLE_0(s.g3.pan_id)
    );
    log::trace!(
        "ipv6 from short addr {:?}",
        network_manager::NetworkManager::ipv6_from_short_addr(s.g3.pan_id, 5)
    );
    // dbg!(config::APP_CONFIG);

    info!("Starting ...");
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());
    let is_coordinator = args
        .nth(0)
        .unwrap_or("true".into())
        .parse()
        .unwrap_or(false);

    log::trace!("Port : {}, coordinator {}", &tty_path, is_coordinator);
    let mut port = serialport::new(tty_path, 460_800)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port {}");

    let (app_tx, app_rx) = flume::unbounded::<Message>();
    // let (usi_tx, usi_rx) = flume::unbounded::<Message>();
    // let (net_tx, net_rx) = flume::unbounded::<adp::Message>();

    // let request = request::AdpInitializeRequest::from_band(message::TAdpBand::ADP_BAND_CENELEC_A);
    let sender = app_tx.clone();
    let mut usi = usi::Port::new(port);
    usi.add_listener(sender);
    let usi_tx = usi.start();

    let cmd_tx = usi_tx.clone();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let network_manager = network_manager::NetworkManager::new(s.g3.pan_id, usi_tx);
            let (tx, rx) = flume::unbounded::<adp::Message>();
            
            log::trace!("Network Manager started ...");
            let t2 = thread::spawn(move || {
                let message_handler: Option<Box<dyn MessageHandler>>;
                if is_coordinator {
                    message_handler = Some(Box::new(coord::Coordinator::new(cmd_tx, tx)));
                } else {
                    message_handler = Some(Box::new(modem::Modem::new(cmd_tx, tx)));
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
            let result = system_tx.send(Message::SystemStartup);
            log::trace!("Sending system startup message result : {:?}", result);
            let system_handle = thread::spawn(move || loop {
                system_tx.send(Message::HeartBeat(SystemTime::now()));
                thread::sleep(TIMER_RESOLUTION);
            });
           
            network_manager.start(rx).await;
        });
        
    // system_handle.join().unwrap();
}

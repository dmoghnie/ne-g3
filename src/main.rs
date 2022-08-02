
mod adp;
mod app_config;
mod common;
mod crc;
mod lbp;
mod lbp_functions;
mod lbp_manager;
mod network_manager;
mod ipv6_frag_manager;
mod request;
mod usi;
mod tun_interface;
mod app_manager;

use std::time::{Duration, SystemTime};
use std::{env, io, str, thread};

use env_logger::Env;
use flume::{Receiver, Sender};


use std::io::{Read, Result as IoResult, Write};
// use crossbeam_channel::{bounded, Sender};


#[macro_use]
extern crate log;
extern crate env_logger;

use log::Level;
use clap::{ArgEnum, Parser};


use crate::app_config::Mode;
use crate::app_manager::AppManager;
use crate::usi::{Message, MessageHandler, OutMessage, UsiSender};

const TIMER_RESOLUTION: Duration = Duration::from_millis(20000);


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// What mode to run the program in
    #[clap(arg_enum)]
    mode: app_config::Mode,

    #[clap(short, long)]
    device: Option<String>,

    #[clap(short, long)]
    speed: Option<u32>,

}




fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let cli = Cli::parse();

    match cli.mode {
        app_config::Mode::Coordinator => env::set_var("NEG3_G3.MODE", "0"),
        app_config::Mode::Modem => env::set_var("NEG3_G3.MODE", "1"),
    }

    if let Some(device_name) = cli.device {
        env::set_var("NEG3_SERIAL.NAME", device_name);
    }
    if let Some(device_speed) = cli.speed {
        env::set_var("NEG3_SERIAL.SPEED", device_speed.to_string());
    }


    let s = app_config::SETTINGS.read().unwrap();

    log::info!("Settings : {:?}", s);


    info!("Starting ...");
    // let mut args = env::args();
    // if let Some(serial_name) = args.nth(1) {
    //     env::set_var("NEG3_SERIAL.NAME", serial_name);
    // }
    // let is_coordinator = args
    //     .nth(0)
    //     .unwrap_or("true".into())
    //     .parse()
    //     .unwrap_or(false);

    let is_coordinator = *app_config::MODE == Mode::Coordinator;
    

    log::info!("Port : {:?}, coordinator {}", *app_config::SERIAL_NAME, is_coordinator);
    let mut port = serialport::new(&(*app_config::SERIAL_NAME), 921_600)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port {}");

    let mut tx_port = port.try_clone().unwrap();
    // let (app_tx, app_rx) = flume::unbounded::<Message>();
    // let (usi_tx, usi_rx) = flume::unbounded::<Message>();
    // let (net_tx, net_rx) = flume::unbounded::<adp::Message>();

    // let request = request::AdpInitializeRequest::from_band(message::TAdpBand::ADP_BAND_CENELEC_A);
    // let sender = app_tx.clone();
    let (app_usi_tx, app_usi_rx) = flume::unbounded::<usi::Message>();
    let mut usi = usi::Port::new(port);
    usi.add_listener(app_usi_tx.clone());
    let (tx, rx) = flume::unbounded::<adp::Message>();
    let usi_tx = usi.start(tx_port);
    let app_manager = AppManager::new(usi_tx.clone(), tx);
    app_manager.start(app_usi_rx, is_coordinator);

   
    
    let network_manager = network_manager::NetworkManager::new(s.g3.pan_id, usi_tx);

    network_manager.start(rx);
    log::info!("Network Manager started ...");
    
    

    
    let system_tx = app_usi_tx.clone();
    let result = system_tx.send(Message::SystemStartup);
    log::info!("Sending system startup message result : {:?}", result);
    let system_handle = thread::spawn(move || loop {
        system_tx.send(Message::HeartBeat(SystemTime::now()));
        thread::sleep(TIMER_RESOLUTION);
    });
    

    system_handle.join().unwrap();
}

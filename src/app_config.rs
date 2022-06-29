use std::sync::RwLock;

use crate::{lbp_functions::{TEapPskKey}, adp::TAdpBand};
use config::Config;


use lazy_static::lazy_static;
use crate::network_manager::NetworkManager;


// pub const PAN_ID: u16 = 0x781D;
// pub const SENDER: [u8; 2] = [0x0, 0x1];
// pub const RECEIVER: [u8; 2] = [0x0, 0x2];
pub const BAND: TAdpBand = TAdpBand::ADP_BAND_CENELEC_A;

// pub const CONF_PSK_KEY: [u8; 16] = [
//     0xab, 0x10, 0x34, 0x11, 0x45, 0x11, 0x1b, 0xc3, 0xc1, 0x2d, 0xe8, 0xff, 0x11, 0x14, 0x22, 0x4,
// ];
pub const CONF_GMK_KEY: [u8; 16] = [
    0xaf, 0x4d, 0x6d, 0xcc, 0xf1, 0x4d, 0xe7, 0xc1, 0xc4, 0x23, 0x5e, 0x6f, 0xef, 0x6c, 0x15, 0x1f,
];

pub const CONF_PSK_KEY_2: [u8; 16] = [
    0xab, 0x10, 0x34, 0x11, 0x45, 0x11, 0x1b, 0xc3, 0xc1, 0x2d, 0xe8, 0xff, 0x11, 0x14, 0x22, 0x3,
];

pub const RAND_S_DEFAULT: [u8; 16] = [0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F];

// Context information table: index 0 (Context 0 with value c_IPv6_PREFIX & x_PAN_ID (length = 80))
// pub const CONF_CONTEXT_INFORMATION_TABLE_0: [u8; 14] = [
//     0x2, 0x0, 0x1, 0x50, 0xfe, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x78, 0x1d,
// ];
pub const CONF_CONTEXT_INFORMATION_TABLE_1: [u8; 10] =
    [0x2, 0x0, 0x1, 0x30, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

pub const X_IDS_ARIB: [u8; 34] = [0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA, 0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA,
0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA, 0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA,
0x53, 0x4D];
pub const X_IDS_CENELEC_FCC: [u8; 8] = [0x81, 0x72, 0x63, 0x54, 0x45, 0x36, 0x27, 0x18];
pub const g_au8CurrGMK:[u8; 16]  = [0xAF, 0x4D, 0x6D, 0xCC, 0xF1, 0x4D, 0xE7, 0xC1, 0xC4, 0x23, 0x5E, 0x6F, 0xEF, 0x6C, 0x15, 0x1F];
pub const g_au8RekeyGMK:[u8; 16] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16];

pub const MAX_HOPS:u8 = 0x0A;

lazy_static! {
    pub static ref G_EAP_PSK_KEY: TEapPskKey = {
        let s = SETTINGS.read().unwrap();
        TEapPskKey(s.g3.psk)
    }
    ;

    // pub static ref APP_CONFIG: Config = {
    //     confy::load("ne-g3").unwrap()
    // };
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new().unwrap());
    pub static ref CONF_CONTEXT_INFORMATION_TABLE_0: [u8; 14] = {
        let s = SETTINGS.read().unwrap();
        NetworkManager::CONF_CONTEXT_INFORMATION_TABLE_0(s.g3.pan_id)
    };
    pub static ref PAN_ID:u16 = {
        let s = SETTINGS.read().unwrap();
        s.g3.pan_id
    };
    pub static ref CONF_PSK_KEY: [u8; 16] = {
        let s = SETTINGS.read().unwrap();
        s.g3.psk
    };
    pub static ref TUN_NAME: String = {
        let s = SETTINGS.read().unwrap();
        s.network.tun.clone()
    };


}

use config::{ConfigError, Environment, File};
use serde_derive::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct G3 {
    pub pan_id: u16,
    pub band: u8,
    pub psk: [u8; 16],
    pub gmk: [u8; 16],
    pub ids: Vec<u8>
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Serial {
    pub name: String,
    pub speed: u32,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub g3: G3,
    pub serial: Serial,
    pub network: Network
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Network {
    pub tun: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("ne-g3.toml"))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(
                File::with_name(&format!("examples/hierarchical-env/config/{}", run_mode))
                    .required(false),
            )
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("examples/hierarchical-env/config/local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            // You may also programmatically change settings
            .set_override("database.url", "postgres://")?
            .build()?;

        // // Now that we're done, let's access our configuration
        // println!("debug: {:?}", s.get_bool("debug"));
        // println!("database: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}
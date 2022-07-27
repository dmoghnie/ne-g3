use std::{sync::RwLock, net::Ipv6Addr};

use crate::{lbp_functions::{TEapPskKey}, adp::{TAdpBand, self, TExtendedAddress}};
use config::Config;


use lazy_static::lazy_static;
use crate::network_manager::NetworkManager;



pub const BAND: TAdpBand = TAdpBand::ADP_BAND_FCC;

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
pub const CONF_CONTEXT_INFORMATION_TABLE_0: [u8; 14] = [
    0x2, 0x0, 0x1, 0x50, 0xfe, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x78, 0x1d,
];
// pub const CONF_CONTEXT_INFORMATION_TABLE_1: [u8; 10] =
//     [0x2, 0x0, 0x1, 0x30, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

pub const CONF_CONTEXT_INFORMATION_TABLE_1: [u8; 12] = [
    0x2, 0x0, 0x1, 0x40, 0xfd, 0x00, 0x0, 0x0, 0x0, 0x2, 0x78, 0x1d,
];

pub const X_IDS_ARIB: [u8; 34] = [0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA, 0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA,
0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA, 0x53, 0x4D, 0xAD, 0xB2, 0xC4, 0xD5, 0xE6, 0xFA,
0x53, 0x4D];
pub const X_IDS_CENELEC_FCC: [u8; 8] = [0x81, 0x72, 0x63, 0x54, 0x45, 0x36, 0x27, 0x18];
pub const g_au8CurrGMK:[u8; 16]  = [0xAF, 0x4D, 0x6D, 0xCC, 0xF1, 0x4D, 0xE7, 0xC1, 0xC4, 0x23, 0x5E, 0x6F, 0xEF, 0x6C, 0x15, 0x1F];
pub const g_au8RekeyGMK:[u8; 16] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16];

pub const MAX_HOPS:u8 = 0x0A;

pub type MacParam = (adp::EMacWrpPibAttribute, u16, Vec<u8>);
pub type AdpParam = (adp::EAdpPibAttribute, u16, Vec<u8>);

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum G3ParamType {
    Adp,
    Mac
}


pub type G3Param = (G3ParamType, u32, u16, Vec<u8>);

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
    // pub static ref CONF_CONTEXT_INFORMATION_TABLE_0: [u8; 14] = {
    //     let s = SETTINGS.read().unwrap();
    //     NetworkManager::CONF_CONTEXT_INFORMATION_TABLE_0(s.g3.pan_id)
    // };
    // pub static ref CONF_CONTEXT_INFORMATION_TABLE_1: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.ula_prefix.clone()
    // };
    pub static ref ULA_NET_PREFIX: [u8; 8] = {
        let s = SETTINGS.read().unwrap();
        s.network.ula_net_prefix
    };
    pub static ref ULA_NET_PREFIX_LEN: u8 = {
        let s = SETTINGS.read().unwrap();
        s.network.ula_net_prefix_len
    };
    pub static ref ULA_HOST_PREFIX: [u8; 6] = {
        let s = SETTINGS.read().unwrap();
        s.network.ula_host_prefix
    };

    pub static ref LOCAL_NET_PREFIX: [u8; 8] = {
        let s = SETTINGS.read().unwrap();
        s.network.local_net_prefix
    };
    pub static ref LOCAL_NET_PREFIX_LEN: u8 = {
        let s = SETTINGS.read().unwrap();
        s.network.local_net_prefix_len
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

    pub static ref COORD_PARAMS: Vec<G3Param> = {
        let params = vec![
            (
                G3ParamType::Mac,
                adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS.into(),
                0,
                vec![0x0, 0x0]
            ),
            (
                G3ParamType::Mac,
                adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID.into(),
                0,
                PAN_ID.to_be_bytes().to_vec()
            ),
            (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(), 0, vec![0x05]),
    
            (
                G3ParamType::Adp,
                adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME.into(),
                0,
                vec![0x10, 0x00]
            ),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_DEFAULT_COORD_ROUTE_ENABLED.into(), 0, vec![0x01]),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(), 0, vec![0xB4, 0x00]),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_DESTINATION_ADDRESS_SET.into(), 0, vec![0xFF, 0x7F]),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_DISABLE_DEFAULT_ROUTING.into(), 0, vec![0x0]),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MAX_HOPS.into(), 0, vec![0x0A]),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY.into(), 0, CONF_PSK_KEY.to_vec()),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 0, CONF_CONTEXT_INFORMATION_TABLE_0.to_vec()),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 1, CONF_CONTEXT_INFORMATION_TABLE_1.to_vec()),
            (
                G3ParamType::Adp,
                adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(),
                0,
                vec![0x5]
            ),
            (
                G3ParamType::Adp,
                adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(),
                0,
                vec![0xB4, 0x00]
            ),
            
    
        ];

        params
    };

    pub static ref MODEM_PARAMS: Vec<G3Param> = {
        let params = vec![
            // (
            //     G3ParamType::Mac,
            //     adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS.into(),
            //     0,
            //     vec![0x0, 0x1]
            // ),
            (
                G3ParamType::Mac,
                adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID.into(),
                0,
                PAN_ID.to_be_bytes().to_vec()
            ),
            (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(), 0, vec![0x05]),
    
            (
                G3ParamType::Adp,
                adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME.into(),
                0,
                vec![0x10, 0x00]
            ),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_DEFAULT_COORD_ROUTE_ENABLED.into(), 0, vec![0x01]),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(), 0, vec![0xB4, 0x00]),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_DESTINATION_ADDRESS_SET.into(), 0, vec![0xFF, 0x7F]),
            // (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_DISABLE_DEFAULT_ROUTING.into(), 0, vec![0x1]),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MAX_HOPS.into(), 0, vec![0x0A]),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY.into(), 0, CONF_PSK_KEY.to_vec()),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 0, CONF_CONTEXT_INFORMATION_TABLE_0.to_vec()),
            (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 1, CONF_CONTEXT_INFORMATION_TABLE_1.to_vec()),
            (
                G3ParamType::Adp,
                adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(),
                0,
                vec![0x5]
            ),
            (
                G3ParamType::Adp,
                adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(),
                0,
                vec![0xB4, 0x00]
            ),
            
           
    
        ];
        params
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
    pub ids: Vec<u8>,

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
    pub network: Network,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Network {
    pub tun: String,
    pub ula_net_prefix: [u8; 8],
    pub ula_host_prefix: [u8; 6],
    pub local_net_prefix: [u8; 8],
    pub ula_net_prefix_len: u8,
    pub local_net_prefix_len: u8
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("ne-g3.toml"))
            .build()?;
        s.try_deserialize()
    }
}

pub fn ula_ipv6_addr_from_pan_id_extended_addr(pan_id: u16, extended_addr: &TExtendedAddress) -> Option<Ipv6Addr> {
    let mut addr = Vec::with_capacity(16);
    addr.extend_from_slice(ULA_NET_PREFIX.as_slice());
    // addr.extend_from_slice(pan_id.to_be_bytes().as_slice());
    addr.extend_from_slice(extended_addr.into());
    if addr.len() == 16 {
        let mut v = [0u8; 16];
        v.copy_from_slice(addr.as_slice());
        Some(Ipv6Addr::from (v))
    }
    else{
        None
    }
}
pub fn ula_ipv6_addr_from_pan_id_short_addr(pan_id: u16, short_addr: u16) -> Option<Ipv6Addr> {
    let mut addr = Vec::with_capacity(16);
    addr.extend_from_slice(ULA_NET_PREFIX.as_slice());
    // addr.extend_from_slice(pan_id.to_be_bytes().as_slice());
    addr.extend_from_slice(ULA_HOST_PREFIX.as_slice());
    addr.extend_from_slice(short_addr.to_be_bytes().as_slice());

    if addr.len() == 16 {
        let mut v = [0u8; 16];
        v.copy_from_slice(addr.as_slice());
        Some(Ipv6Addr::from (v))
    }
    else{
        None
    }
}
pub fn local_ipv6_add_from_pan_id_short_addr(pan_id: u16, short_addr: u16) -> Option<Ipv6Addr> {
    let mut addr = Vec::with_capacity(16);
    addr.extend_from_slice(LOCAL_NET_PREFIX.as_slice());
    addr.extend_from_slice(pan_id.to_be_bytes().as_slice());
    // Ipv6Addr::new(0xfe80, 0x0, 0x0, 0x0, pan_id, 0x00ff, 0xfe00, short_addr)
    addr.extend_from_slice(&[0x00, 0xff, 0xfe, 0x00]);
    addr.extend_from_slice(short_addr.to_be_bytes().as_slice());

    if addr.len() == 16 {
        let mut v = [0u8; 16];
        v.copy_from_slice(addr.as_slice());
        Some(Ipv6Addr::from (v))
    }
    else{
        None
    }    
}

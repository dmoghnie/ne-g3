use std::{sync::RwLock, net::Ipv6Addr};

use crate::{lbp_functions::{TEapPskKey}, adp::{TAdpBand, self, TExtendedAddress}};
use config::Config;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;


use lazy_static::lazy_static;
use serde::Serialize;
use crate::network_manager::NetworkManager;



#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ArgEnum, Debug, 
    TryFromPrimitive, IntoPrimitive, Deserialize)]
#[repr(u8)]
pub enum Mode{
    Coordinator = 0u8,
    Modem
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum G3ParamType {
    Adp,
    Mac
}


pub type G3Param = (G3ParamType, u32, u16, Vec<u8>);

lazy_static! {
    

    // pub static ref APP_CONFIG: Config = {
    //     confy::load("ne-g3").unwrap()
    // };
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new("").unwrap());
    // pub static ref CONF_CONTEXT_INFORMATION_TABLE_0: [u8; 14] = {
    //     let s = SETTINGS.read().unwrap();
    //     NetworkManager::CONF_CONTEXT_INFORMATION_TABLE_0(s.g3.pan_id)
    // };
    // pub static ref CONF_CONTEXT_INFORMATION_TABLE_1: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.ula_prefix.clone()
    // };

    // pub static ref MODE : Mode = {
    //     let s = SETTINGS.read().unwrap();
    //     Mode::try_from_primitive (s.g3.mode).unwrap().clone()
        
    // };
    // pub static ref SERIAL_SPEED: u32 = {
    //     let s = SETTINGS.read().unwrap();
    //     s.serial.speed
    // };
    // pub static ref G_EAP_PSK_KEY: TEapPskKey = {
    //     let s = SETTINGS.read().unwrap();
    //     TEapPskKey(s.g3.psk)
    // };
    // pub static ref X_IDS_ARIB: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.ids_arib.clone()
    // };
    // pub static ref X_IDS_CENELEC_FCC: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.ids_cenelec_fcc.clone()
    // };
    // pub static ref BAND: TAdpBand = {
    //     let s = SETTINGS.read().unwrap();
    //     TAdpBand::try_from_primitive(s.g3.band).unwrap().clone()
    // };
    // pub static ref MAX_HOPS: u8 = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.max_hops
    // };
    // pub static ref REKEY_GMK: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.rekey_gmk.clone()
    // };

    // pub static ref GMK: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.gmk.clone()
    // };
    // pub static ref SERIAL_NAME: String = {
    //     let s = SETTINGS.read().unwrap();
    //     s.serial.name.clone()
    // };
    // pub static ref ULA_NET_PREFIX: [u8; 8] = {
    //     let s = SETTINGS.read().unwrap();
    //     s.network.ula_net_prefix
    // };
    // pub static ref ULA_NET_PREFIX_LEN: u8 = {
    //     let s = SETTINGS.read().unwrap();
    //     s.network.ula_net_prefix_len
    // };
    // pub static ref ULA_HOST_PREFIX: [u8; 6] = {
    //     let s = SETTINGS.read().unwrap();
    //     s.network.ula_host_prefix
    // };

    // pub static ref LOCAL_NET_PREFIX: [u8; 8] = {
    //     let s = SETTINGS.read().unwrap();
    //     s.network.local_net_prefix
    // };
    // pub static ref LOCAL_NET_PREFIX_LEN: u8 = {
    //     let s = SETTINGS.read().unwrap();
    //     s.network.local_net_prefix_len
    // };
    // pub static ref PAN_ID:u16 = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.pan_id
    // };
    // pub static ref CONF_PSK_KEY: [u8; 16] = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.psk
    // };
    // pub static ref CONF_PSK_2_KEY: [u8; 16] = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.psk_2
    // };
    // pub static ref CONF_CONTEXT_INFORMATION_TABLE_0: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.context_information_table_0.clone()
    // };
    // pub static ref CONF_CONTEXT_INFORMATION_TABLE_1: Vec<u8> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.g3.context_information_table_1.clone()
    // };
    // pub static ref TUN_NAME: Option<String> = {
    //     let s = SETTINGS.read().unwrap();
    //     s.network.tun.clone()
    // };

    // pub static ref COORD_PARAMS: Vec<G3Param> = {
    //     let params = vec![
    //         (
    //             G3ParamType::Mac,
    //             adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID.into(),
    //             0,
    //             PAN_ID.to_be_bytes().to_vec()
    //         ),
    //         (
    //             G3ParamType::Mac,
    //             adp::EMacWrpPibAttribute::MAC_WRP_PIB_KEY_TABLE.into(),
    //             0,
    //             GMK.to_vec()
    //         ),
    //         //TODO rekey 
    //         (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(), 0, vec![0x05]), //TODO, parameterize
    //         (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_ACTIVE_KEY_INDEX.into(), 0, vec![0x00]), //TODO parameterize
    
    //         (
    //             G3ParamType::Adp,
    //             adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME.into(),
    //             0,
    //             vec![0x10, 0x00]
    //         ),
            
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MAX_HOPS.into(), 0, vec![*MAX_HOPS]),
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY.into(), 0, CONF_PSK_KEY.to_vec()),
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 0, CONF_CONTEXT_INFORMATION_TABLE_0.to_vec()),
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 1, CONF_CONTEXT_INFORMATION_TABLE_1.to_vec()),
            
    //         (
    //             G3ParamType::Adp,
    //             adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(),
    //             0,
    //             vec![0xB4, 0x00]
    //         ),
    //         (
    //             G3ParamType::Mac,
    //             adp::EMacWrpPibAttribute::MAC_WRP_PIB_SHORT_ADDRESS.into(),
    //             0,
    //             vec![0x0u8, 0x0u8]
    //         ),
            
    
    //     ];

    //     params
    // };

    // pub static ref MODEM_PARAMS: Vec<G3Param> = {
    //     let params = vec![

    //         (
    //             G3ParamType::Mac,
    //             adp::EMacWrpPibAttribute::MAC_WRP_PIB_PAN_ID.into(),
    //             0,
    //             PAN_ID.to_be_bytes().to_vec()
    //         ),
    //         (G3ParamType::Adp, adp::EAdpPibAttribute::ADP_IB_SECURITY_LEVEL.into(), 0, vec![0x05]),
    
    //         (
    //             G3ParamType::Adp,
    //             adp::EAdpPibAttribute::ADP_IB_MAX_JOIN_WAIT_TIME.into(),
    //             0,
    //             vec![0x10, 0x00]
    //         ),
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MAX_HOPS.into(), 0, vec![*MAX_HOPS]),
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_MANUF_EAP_PRESHARED_KEY.into(), 0, CONF_PSK_KEY.to_vec()),
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 0, CONF_CONTEXT_INFORMATION_TABLE_0.to_vec()),
    //         (G3ParamType::Adp,adp::EAdpPibAttribute::ADP_IB_CONTEXT_INFORMATION_TABLE.into(), 1, CONF_CONTEXT_INFORMATION_TABLE_1.to_vec()),
    //         (
    //             G3ParamType::Adp,
    //             adp::EAdpPibAttribute::ADP_IB_ROUTING_TABLE_ENTRY_TTL.into(),
    //             0,
    //             vec![0xB4, 0x00]
    //         ),
            
           
    
    //     ];
    //     params
    // };

}

use config::{ConfigError, Environment, File};
use serde_derive::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct G3 {
    pub mode: u8,
    pub pan_id: u16,
    pub band: u8,
    pub psk: [u8; 16],
    pub psk_2: [u8; 16],
    pub gmk: Vec<u8>,
    pub rekey_gmk: Vec<u8>,
    pub ids: Vec<u8>,
    pub context_information_table_0: Vec<u8>,
    pub context_information_table_1: Vec<u8>,
    pub ids_arib: Vec<u8>,
    pub ids_cenelec_fcc: Vec<u8>,
    pub max_hops: u8
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Serial {
    pub name: String,
    pub speed: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub g3: G3,
    pub serial: Serial,
    pub network: Network,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Network {
    pub tun: Option<String>,
    pub ula_net_prefix: [u8; 8],
    pub ula_host_prefix: [u8; 6],
    pub local_net_prefix: [u8; 8],
    pub ula_net_prefix_len: u8,
    pub local_net_prefix_len: u8
}

impl Settings {
    pub fn new(file_name: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(file_name))
            .add_source(Environment::with_prefix("NEG3"))
            .build()?;
        s.try_deserialize()
    }
}

pub fn ula_ipv6_addr_from_pan_id_extended_addr(ula_net_prefix: &[u8], pan_id: u16, extended_addr: &TExtendedAddress) -> Option<Ipv6Addr> {
    let mut addr = Vec::with_capacity(16);
    addr.extend_from_slice(ula_net_prefix);
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
pub fn ula_ipv6_addr_from_pan_id_short_addr(ula_net_prefix: &[u8], ula_host_prefix: &[u8], 
        pan_id: u16, short_addr: u16) -> Option<Ipv6Addr> {
    let mut addr = Vec::with_capacity(16);
    addr.extend_from_slice(ula_net_prefix);
    // addr.extend_from_slice(pan_id.to_be_bytes().as_slice());
    addr.extend_from_slice(ula_host_prefix);
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
pub fn local_ipv6_add_from_pan_id_short_addr(local_net_prefix: &[u8], pan_id: u16, short_addr: u16) -> Option<Ipv6Addr> {
    let mut addr = Vec::with_capacity(16);
    addr.extend_from_slice(local_net_prefix);
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

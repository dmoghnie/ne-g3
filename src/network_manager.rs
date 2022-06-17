use std::{
    intrinsics::transmute,
    net::{Ipv4Addr, Ipv6Addr},
    vec, thread,
};

use bytes::BytesMut;
use packet::ip::v6::Packet;

use crate::{adp::{self, EAdpStatus}, usi};

use tun::{platform::{Queue, posix}, Device, IntoAddress};
#[cfg(target_os = "macos")]

pub struct NetworkManager {
    cmd_tx: flume::Sender<usi::Message>,
    tun_reader: Option<posix::Reader>,
    tun_writer: Option<posix::Writer>,
}
/*
By design, the G3-PLC protocol stack allows native support of the IPv6 protocol, which grants end-user flexibility to fulfil business requirements when choosing the appropriate higher layers (ISO/OSI transport and application layers). This key feature also secures G3-PLC infrastructures in the long term, thanks to the scalability and future application compatibility provided by IPv6.
An IPv6 address is a 128-bit identifier (16 Bytes) that allows to uniquely identify a device in a network. It is made up of two parts:
- The 64-bit subnet prefix, which identifies the network the device belongs to.
- The 64-bit interface identifier, which identifies the device itself.
In a G3-PLC network, interface identifiers are always formatted as follows:
yyyy:00ff:fe00:xxxx (hexadecimal representation)
where yyyy corresponds to the PAN identifier (PAN-ID) and xxxx to the device’s short address (see
§3.3).
Several types of IPv6 subnet prefixes are available for use in an IPv6 network:
- Link-local prefix: the support of this prefix is mandatory for any IPv6 device and is always available without configuration. Yet link-local addresses cannot offer end-to-end IPv6 connectivity from a host located outside of the local network.
The link local prefix always equals:
fe80:0000:0000:0000 (hexadecimal representation)

*/
impl NetworkManager {
    pub fn new(cmd_tx: flume::Sender<usi::Message>) -> Self {
        NetworkManager {
            cmd_tx: cmd_tx,
            tun_reader: None,
            tun_writer: None,
            
        }
    }
    pub fn ipv4_from_short_addr(short_addr: u16) -> Ipv4Addr {
        let b = short_addr.to_be_bytes();
        Ipv4Addr::new(10u8, 0u8, b[0], b[1])
    }
    pub fn ipv6_from_short_addr(pan_id: u16, short_addr: u16) -> Ipv6Addr {
        Ipv6Addr::new(0xfe80, 0x0, 0x0, 0x0, pan_id, 0x00ff, 0xfe00, short_addr)
    }
    pub fn pan_id_and_short_addr_from_ipv6(ipv6: &Ipv6Addr) -> (u16, u16) {
        let segments = ipv6.segments();
        (segments[4], segments[7])
    }
    pub fn CONF_CONTEXT_INFORMATION_TABLE_0(pan_id: u16) -> [u8; 14] {
        let mut v = [
            0x2, 0x0, 0x1, 0x50, 0xfe, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff,
        ];
        let b = pan_id.to_be_bytes();
        v[12] = b[0];
        v[13] = b[1];
        v
    }
    fn process_adp_message(&mut self, msg: &adp::Message) {
        match msg {
            adp::Message::AdpG3DataEvent(packet) => {}
            adp::Message::AdpG3NetworkStartResponse(network_start_response) => {
                if network_start_response.status == EAdpStatus::G3_SUCCESS {
                    self.start_tun(0);
                }
            }
            adp::Message::AdpG3NetworkJoinResponse(network_join_response) => {
                if network_join_response.status == EAdpStatus::G3_SUCCESS {
                    self.start_tun(network_join_response.network_addr);
                }
            }
            _ => {}
        }
    }
    fn start_tun(&mut self, short_addr: u16) -> Option<(posix::Reader, posix::Writer)> {
        let ipv4 = Self::ipv4_from_short_addr(short_addr);
        let mut config = tun::Configuration::default();

        config.address(&ipv4).netmask((255, 255, 0, 0)).up();
        tun::create(&config).map_or(None, |v| Some(v.split()))        
    }
    pub fn start(mut self) -> flume::Sender<adp::Message> {
        let (rx, tx) = flume::unbounded::<adp::Message>();
        thread::spawn (move || {
            match tx.recv() {
                Ok(msg) => {
                    self.process_adp_message(&msg);
                },
                Err(e) => {
                    log::warn!("NetworkManager error receiving adp message {}", e);
                },
            }
        });
        rx
    }
}

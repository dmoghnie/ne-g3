use std::{
    collections::HashMap,
    intrinsics::transmute,
    io::{Error, Read},
    net::{Ipv4Addr, Ipv6Addr},
    process::Command,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, sleep, sleep_ms},
    time::Duration,
    vec,
};

use byteorder::{NativeEndian, WriteBytesExt, NetworkEndian};
use config::Config;
use pnet_packet::{
    ipv4::{Ipv4Flags, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    Packet,
};

use crate::{app_config, tun_interface::TunInterface, adp::{TExtendedAddress, EAdpPibAttribute}, request::AdpSetRequest, lbp_manager, lbp};
use std::sync::atomic::Ordering;
use crate::ipv6_frag_manager;
use crate::request;


use crate::{
    adp::{self, EAdpStatus},
    request::AdpDataRequest,
    usi,
};

use rand::Rng;

enum PacketProtocol {
    IPv4,
    IPv6,
    Other(u8),
}
fn infer_proto(buf: &[u8]) -> PacketProtocol {
    match buf[0] >> 4 {
        4 => PacketProtocol::IPv4,
        6 => PacketProtocol::IPv6,
        p => PacketProtocol::Other(p),
    }
}
fn cmd(program: &str, cmd: &str, args: &[&str]) {
    let ecode = Command::new(program)
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}

#[derive(Debug)]
enum TunPayload {
    Data(Vec<u8>),
    Stop,
    Error(()), //TODO
}

#[derive(Debug)]
struct TunMessage {
    short_addr: u16,
    payload: TunPayload,
}

impl TunMessage {
    pub fn new(short_addr: u16, payload: TunPayload) -> Self {
        TunMessage {
            short_addr,
            payload,
        }
    }
    pub fn get_payload(self) -> TunPayload {
        self.payload
    }
    pub fn get_short_addr(self) -> u16 {
        self.short_addr
    }
}

struct TunDevice {
    short_addr: u16,
    listener: flume::Sender<TunMessage>,
}

impl TunDevice {
    pub fn new(short_addr: u16, listener: flume::Sender<TunMessage>) -> Self {
        TunDevice {
            short_addr,
            listener,
        }
    }


    
    pub fn start(self, short_addr: u16, mut rx: flume::Receiver<TunMessage>, extended_addr: &Option<TExtendedAddress>) {
        use std::{thread::sleep, time::Duration, io::Read, io::Write};


        use crate::app_config::PAN_ID;
        /*fd00:0:2:781d:1122:3344:5566:1 */
        
        let local_link = app_config::local_ipv6_add_from_pan_id_short_addr(*PAN_ID, short_addr).unwrap();
        let mut ula: Option<Ipv6Addr> = None;
        // ula = app_config::ula_ipv6_addr_from_pan_id_short_addr(*PAN_ID, short_addr);
        if let Some(extended_addr) = extended_addr {            
            ula = app_config::ula_ipv6_addr_from_pan_id_extended_addr(*PAN_ID, extended_addr);
        }

        let tun_interface = TunInterface::new().unwrap();

        cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
        
        cmd("ip",
            "ip",
            &[
                "addr",
                "add",
                "dev",
                tun_interface.name(),
                &format!("{}/{}", local_link, *app_config::LOCAL_NET_PREFIX_LEN),
            ],
        );
        if let Some(ula) = ula {
        cmd(
            "ip",
            "ip",
            &["addr", "add", "dev", tun_interface.name(), &format!("{}/{}", ula, *app_config::ULA_NET_PREFIX_LEN)],
        );
    }
        
        cmd("ip", "ip", &["link", "set", "up", "dev", tun_interface.name()]);
    }
    else if #[cfg(target_os = "macos")] {

        cmd("ifconfig",
        "ifconfig",
        &[
            tun_interface.name(),
            "inet6",            
            &format!("{}/{}", local_link, *app_config::LOCAL_NET_PREFIX_LEN)
        ]);
        if let Some(ula) = ula {
        cmd(
            "ifconfig",
            "ifconfig",
            &[tun_interface.name(), "inet6", &format!("{}/{}", ula, *app_config::ULA_NET_PREFIX_LEN)],
        );
    }

        cmd("ifconfig", "ifconfig", &[tun_interface.name(), "up"]);
    }
    }   

        cmd("ifconfig", "ifconfig", &[tun_interface.name(), "mtu", "1280"]);
        let iface = Arc::new(tun_interface);
        let iface_writer = iface.clone();
        let iface_reader = iface.clone();

        #[cfg(target_os = "linux")]
        let skip = 0usize;
        #[cfg(target_os = "macos")]
        let skip = 4usize;
        thread::spawn(move || {
            let mut buf = vec![0u8; 2048];
            loop {
                match iface_reader.recv(&mut buf) {
                    Ok(size) => {
                        log::info!("tun received {} bytes", size);
                        if size > 0 {
                            match infer_proto(&buf[skip..]) {
                                PacketProtocol::IPv4 => {
                                    log::warn!("Protocol IPV4 not implemented yet");
                                }
                                PacketProtocol::IPv6 => {
                                    // let packet = Ipv6Packet::new(&buf[..size])
                                    // .unwrap();
                                    // let pkts = ipv6_frag_manager::fragment_packet(packet, 1280);
                                    // log::info!("Tun message fragmented into {} packets ", pkts.len());
                                    // for pkt in pkts{
                                        
                                        match self.listener.send(TunMessage::new(
                                            self.short_addr,
                                            TunPayload::Data(buf[skip..size].to_vec()),
                                        )) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                log::warn!(
                                                    "failed to send TunMessage to listener {}",
                                                    e
                                                )
                                            }
                                        }
                                        // sleep(Duration::from_millis(10));
                                    // }

                                }
                                PacketProtocol::Other(_) => {}
                            }
                        }
                    }
                    Err(e) => log::warn!("failed to read data from TUN : {}", e),
                }
                // sleep(Duration::from_millis(10));
            }
        });

        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(tun_msg) => {
                        match tun_msg.get_payload() {
                            TunPayload::Data(data) => {
                                log::debug!("TUN interface sending Packet {:?}", data);
                                match iface_writer.send(&data) {
                                    Ok(size) => {
                                        log::info!("TUN interface wrote {} bytes", size)
                                    }
                                    Err(e) => {
                                        log::warn!("TUN interface failed to write data : {}", e)
                                    }
                                }
                            }
                            TunPayload::Stop => {
                                log::warn!("implment TunPayload::Stop");
                            }
                            TunPayload::Error(_) => {
                                log::warn!("Tun payload error");
                            }
                        }
                        // socket.send_slice(tun_msg.get_payload())
                    }
                    Err(e) => {}
                }
            }
        });
    }
}

pub struct NetworkManager {
    pan_id: u16,
    cmd_tx: flume::Sender<usi::Message>,
    tun_devices: HashMap<u16, flume::Sender<TunMessage>>,
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
    pub fn new(pan_id: u16, cmd_tx: flume::Sender<usi::Message>) -> Self {
        NetworkManager {
            pan_id,
            cmd_tx: cmd_tx,
            tun_devices: HashMap::new(),
        }
    }
    pub fn ipv6_is_unicast_link_local(addr: &Ipv6Addr) -> bool {
        (addr.segments()[0] & 0xffc0) == 0xfe80
    }
    pub fn ipv4_from_short_addr(short_addr: u16) -> Ipv4Addr {
        let s = short_addr + 1;
        let b = s.to_be_bytes();
        Ipv4Addr::new(10u8, 1u8, b[0], b[1]) //TODO parameterize
    }
    pub fn short_addr_from_ipv4(ipv4: &Ipv4Addr) -> u16 {
        let o = ipv4.octets();
        let s = ((o[2] as u16) << 8) | (o[3] as u16);
        s - 1
    }

    pub fn ipv4_addr_from_ipv6(ipv6_addr: Ipv6Addr) -> Ipv4Addr {
        let (pan_id, short_addr) = Self::pan_id_and_short_addr_from_ipv6(&ipv6_addr);
        Self::ipv4_from_short_addr(short_addr)
    }
    pub fn pan_id_and_short_addr_from_ipv6(ipv6: &Ipv6Addr) -> (u16, u16) {
        log::info!("---> pan_id_and_short_addr_from_ipv6 : {} ", ipv6);
        let segments = ipv6.segments();
        log::info!("---> pan_id_and_short_addr_from_ipv6 : {:?} ", segments);
        (segments[4], segments[7])
    }
    pub fn short_addr_from_ipv6(ipv6: &Ipv6Addr) -> u16 {
        
        let segments = ipv6.segments();
        
        ipv6.segments()[7]
    }
    pub fn dscp_ecn_to_traffic_class(dscp: u8, ecn: u8) -> u8 {
        /*
        Traffic Class (8-bits): These 8 bits are divided into two parts. The most significant 6 bits are used for Type of Service to
        let the Router Known what services should be provided to this packet. The least significant 2 bits are used for
        Explicit Congestion Notification (ECN).
         */
        dscp << 2 | (ecn & 0b1111_1100)
    }
    pub fn traffic_class_to_dscp_ecn(traffic_class: u8) -> (u8, u8) {
        (traffic_class >> 2, traffic_class & 0b0000_0011)
    }

    fn get_extended_address_from_short_addr(short_addr: u16) -> [u8; 8] {
        let mut v = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x0, 0x0,
        ];
        let b = short_addr.to_be_bytes();
        v[6] = b[0];
        v[7] = b[1];
        v
    } 
    fn ipv6_to_tun_payload_and_short_addr(buf: &Vec<u8>) -> Option<(TunPayload, u16)> {
        let mut ipv6_pkt = Ipv6Packet::new(buf)?;
        let (_, short_addr) = Self::pan_id_and_short_addr_from_ipv6(&ipv6_pkt.get_destination());
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                return Some((TunPayload::Data(buf.to_vec()), short_addr));
            } else if #[cfg(target_os = "macos")] {
                let mut v = Vec::<u8>::with_capacity(buf.len() + 4);
                v.write_u16::<NativeEndian>(0).unwrap();
                v.write_u16::<NetworkEndian>(libc::PF_INET6 as u16).unwrap();
                v.extend_from_slice(buf);
        
               return Some((TunPayload::Data(v), short_addr));
            }
        }
        None
    }



    pub fn start(mut self, mut rx: flume::Receiver<adp::Message>) {
        let (tun_tx, mut tun_rx) = flume::unbounded::<TunMessage>();
        log::info!("network manager starting ...");

        let mut extended_addr :Option<TExtendedAddress> =  None;

        thread::spawn(move || {
            let mut buffer_available = true;
            let mut lbp_manager = lbp_manager::LbpManager::new();
            let mut current_short_dst_addr:Option<u16> = None;
            let mut current_out_msg : Option<Vec<u8>> = None;

            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        log::debug!("Network manager received {:?}", msg);
                        match msg {
                            adp::Message::AdpG3DataEvent(g3_data) => {
                                log::info!("Network manager received data  {} bytes", g3_data.nsdu.len());
                                if let Some((payload, short_addr)) =
                                    Self::ipv6_to_tun_payload_and_short_addr(&g3_data.nsdu)
                                {
                                    if let Some(tx) = self.tun_devices.get(&short_addr) {
                                        log::info!("found sender for short addr {} -- sending TunPayload::Data", short_addr);
                                        match tx.send(TunMessage {
                                            short_addr: short_addr,
                                            payload: payload,
                                        }) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                log::warn!(
                                                    "Failed to send ipv4 packet to TUN : {}",
                                                    e
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                            adp::Message::AdpG3GetMacResponse(response) => {
                                if let Ok(attr) = adp::EMacWrpPibAttribute::try_from(response.attribute_id) {
                                    match attr {
                                        adp::EMacWrpPibAttribute::MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS => {
                                            let mut v = response.attribute_val.clone();
                                            v.reverse();
                                            extended_addr = 
                                                TExtendedAddress::try_from(v.as_slice()).map_or(None, |v| Some(v));
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            adp::Message::AdpG3NetworkStartResponse(network_start_response) => {
                                if network_start_response.status == EAdpStatus::G3_SUCCESS {
                                    let short_addr = 0u16; //TODO, get the actual address from configuration
                                    log::info!("received network start response, starting interface for address {}", short_addr);
                                    if self.tun_devices.contains_key(&short_addr) {
                                        log::warn!(
                                            "Received network start for device already started"
                                        );
                                    } else {
                                        let tun_device = TunDevice::new(short_addr, tun_tx.clone());
                                        let (tx, mut rx) = flume::unbounded::<TunMessage>();
                                        self.tun_devices.insert(short_addr, tx);

                                        
                                        tun_device.start(short_addr, rx, &extended_addr);
                                    }
                                }
                                else{
                                    log::error!("Failed to start network {:?}", network_start_response.status);
                                }
                            }
                            adp::Message::AdpG3NetworkJoinResponse(network_join_response) => {
                                if network_join_response.status == EAdpStatus::G3_SUCCESS {
                                    let short_addr = network_join_response.network_addr;
                                    if self
                                        .tun_devices
                                        .contains_key(&network_join_response.network_addr)
                                    {
                                        log::warn!("Received network join response for address : {}, while device already in list", short_addr);
                                    } else {
                                        let tun_device = TunDevice::new(short_addr, tun_tx.clone());
                                        let (tx, mut rx) = flume::unbounded::<TunMessage>();
                                        self.tun_devices.insert(short_addr, tx);

                                        tun_device.start(short_addr, rx, &extended_addr);
                                    }
                                }
                            }
                            adp::Message::AdpG3BufferEvent(event) => {
                                log::info!("Received buffer ready : {}", event.buffer_ready);
                                buffer_available = event.buffer_ready;
                            }
                            adp::Message::AdpG3LbpEvent(lbp_event) => {
                                if let Some(lbp_message) = lbp::adp_message_to_lbp_message(&lbp_event) {
                                    log::info!("Received lbp_event {:?}", lbp_message);
                                    if let Some(result) = lbp_manager.process_msg(&lbp_message) {
                                        self.cmd_tx.send(usi::Message::UsiOut(result.into()));
                                    }
                                }
                            }
                            adp::Message::AdpG3LbpReponse(lbp_response) => {
                                lbp_manager.process_response (&lbp_response);
                            }
                            adp::Message::AdpG3SetResponse(resp) => {
                                if let Ok(attr) = EAdpPibAttribute::try_from(resp.attribute_id) {
                                    if attr == EAdpPibAttribute::ADP_IB_MANUF_IPV6_ULA_DEST_SHORT_ADDRESS {
                                        if let Some(ref pkt) = current_out_msg {
                                            let data_request = AdpDataRequest::new(
                                                rand::thread_rng().gen(),
                                                &pkt,
                                                true,
                                                0,
                                            );
                                            
                                            match self.cmd_tx.send(usi::Message::UsiOut(data_request.into())) {
                                                Ok(_) => {log::info!("Send to usi ")},
                                                Err(e) => {log::warn!("Failed to send to usi {}", e)},
                                            }                                            
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {}
                }
                 if buffer_available {
                    match tun_rx.try_recv() {
                        Ok(msg) => {
                            match msg.payload {
                                TunPayload::Data(pkt) => {
                                    // log::info!("send {} bytes to G3", pkt.len());

                                    if let Some(ipv6) = Ipv6Packet::new (&pkt) {
                                        log::info!("Packet {:?}", ipv6);
                                        let dst_addr = ipv6.get_destination();
                                        if !Self::ipv6_is_unicast_link_local(&dst_addr) {
                                            let short_addr = dst_addr.segments()[7];
                                            // if let Some(short_addr) = lbp_manager.get_short_addr_from_ipv6_addr(dst_addr) {                                                
                                            //     let v = short_addr.to_le_bytes().to_vec();
                                            //     log::info!("Setting short addr for packet destination {} : {}", dst_addr, short_addr);
                                            //     current_out_msg = Some(pkt);
                                            //     let request = AdpSetRequest::new(EAdpPibAttribute::ADP_IB_MANUF_IPV6_ULA_DEST_SHORT_ADDRESS, 0, &v);
                                            //     self.cmd_tx.send(usi::Message::UsiOut(request.into()));
                                            // }               
                                            let v = short_addr.to_le_bytes().to_vec();
                                            log::info!("Setting short addr for packet destination {} : {} : {:?}", dst_addr, short_addr, v);
                                            current_out_msg = Some(pkt);
                                            let request = AdpSetRequest::new(EAdpPibAttribute::ADP_IB_MANUF_IPV6_ULA_DEST_SHORT_ADDRESS, 0, &v);
                                            self.cmd_tx.send(usi::Message::UsiOut(request.into()));                             
                                        }
                                        else{
// ipv6.set_src_addr(Self::ipv6_from_short_addr(*app_config::PAN_ID, msg.short_addr).into());
                                    //  log::info!("ipv6 pkt : {:?}", pkt);
                                    let data_request = AdpDataRequest::new(
                                        rand::thread_rng().gen(),
                                        &pkt,
                                        true,
                                        0,
                                    );
                                    
                                    match self.cmd_tx.send(usi::Message::UsiOut(data_request.into())) {
                                        Ok(_) => {log::info!("Send to usi ")},
                                        Err(e) => {log::warn!("Failed to send to usi {}", e)},
                                    }
                                        }
                                    }
                                    
                                    
                                }
                                TunPayload::Stop => { //Should we use this as a notification that the device is stopped or should we have a separate message
                                }
                                TunPayload::Error(e) => {
                                    log::info!("Received error from device");
                                }
                            }
                        }
                        Err(_) => {}
                    }

                 }
                 else {
                    // tun_rx.drain();
                }
                sleep(Duration::from_millis(10)); //TODO, spin threads and recv instead of try_recv
            }
        });
    }
}

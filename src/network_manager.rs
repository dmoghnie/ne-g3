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


struct TunDevice {    
    listener: flume::Sender<TunPayload>,
}

impl TunDevice {
    pub fn new(listener: flume::Sender<TunPayload>) -> Self {
        TunDevice {           
            listener,
        }
    }


    
    pub fn start(self, buffers_available: Arc<AtomicBool>, settings: &app_config::Settings, short_addr: u16, 
        mut rx: flume::Receiver<TunPayload>, extended_addr: &Option<TExtendedAddress>) {
        use std::{thread::sleep, time::Duration, io::Read, io::Write};


        /*fd00:0:2:781d:1122:3344:5566:1 */
        
        let local_link = app_config::local_ipv6_add_from_pan_id_short_addr(&settings.network.local_net_prefix, settings.g3.pan_id, short_addr).unwrap();
        let mut ula: Option<Ipv6Addr> = None;
        ula = app_config::ula_ipv6_addr_from_pan_id_short_addr(&settings.network.ula_net_prefix, 
            &settings.network.ula_host_prefix, settings.g3.pan_id, short_addr);
        // if let Some(extended_addr) = extended_addr {            
        //     ula = app_config::ula_ipv6_addr_from_pan_id_extended_addr(*PAN_ID, extended_addr);
        // }

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
                &format!("{}/{}", local_link, settings.network.local_net_prefix_len),
            ],
        );
        if let Some(ula) = ula {
        cmd(
            "ip",
            "ip",
            &["addr", "add", "dev", tun_interface.name(), &format!("{}/{}", ula, settings.network.ula_net_prefix_len)],
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
            &format!("{}/{}", local_link, settings.network.local_net_prefix_len)
        ]);
        if let Some(ula) = ula {
        cmd(
            "ifconfig",
            "ifconfig",
            &[tun_interface.name(), "inet6", &format!("{}/{}", ula, settings.network.ula_net_prefix_len)],
        );
    }

        cmd("ifconfig", "ifconfig", &[tun_interface.name(), "up"]);
    }
    }   


        //TODO new linux distributions don't have ifconfig. install net-tools or fix by using ip command
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
                        if size > 0 && buffers_available.load(Ordering::SeqCst){
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
                                        log::trace!("--> tun {:?}", buf);
                                        match self.listener.send(
                                            TunPayload::Data(buf[skip..size].to_vec())) {
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
                        match tun_msg {
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
    cmd_tx: flume::Sender<usi::Message>,    
    buffers_available: Arc<AtomicBool>,
    tun_tx: Option<flume::Sender<TunPayload>>
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
impl <'a> NetworkManager {
    pub fn new(settings: &'a app_config::Settings, cmd_tx: flume::Sender<usi::Message>) -> Self {
        NetworkManager { 
            buffers_available: Arc::new(AtomicBool::new(true)),           
            cmd_tx: cmd_tx,                        
            tun_tx: None
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



    pub fn start(mut self, settings: &'a app_config::Settings, mut rx: flume::Receiver<adp::Message>) {
        let (tun_tx, mut tun_rx) = flume::unbounded::<TunPayload>();
        log::info!("network manager starting ...");

        let mut extended_addr :Option<TExtendedAddress> =  None;

        let settings = settings.clone();

        thread::spawn(move || {

            let mut lbp_manager = lbp_manager::LbpManager::new(&settings.g3);
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
                                    if let Some(ref tx) = self.tun_tx {
                                       
                                        match tx.send(payload) {
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
                            // The reason why we don't just use the configured coord address.
                            // Our requirements dictate that a node can switch from device mode to coord mode (distributed model which will be developed later)
                            // as part of peer to peer, self healing network.
                            // Obviously a lot of work has to be done in coordinating the multiple device/coordinators operating in concert 
                            // (not sure if this is possible in the current G3 PLC standard or a limitation in Microship's stack implementation).
                            // more layers for distributed database has to be added.
                            adp::Message::AdpG3GetResponse(response) =>{
                                if let Ok(attr) = adp::EAdpPibAttribute::try_from(response.attribute_id) {
                                    match attr {
                                        adp::EAdpPibAttribute::ADP_IB_COORD_SHORT_ADDRESS => {
                                            let v = response.attribute_val;
                                            if v.len() == 2 {
                                                let coord_short_addr = u16::from_be_bytes([v[0], v[1]]);
                                                let tun_device = TunDevice::new (tun_tx.clone());
                                                let (tx, mut rx) = flume::unbounded::<TunPayload>();
                                                lbp_manager.set_short_addr(coord_short_addr);
                                                self.tun_tx = Some(tx);
                                                
                                                tun_device.start(self.buffers_available.clone(), &settings, 
                                                    coord_short_addr, rx, &extended_addr);
                                            }
                                        }
                                        _ => {

                                        }
                                    }
                                }                                
                            }
                        
                            adp::Message::AdpG3NetworkStartResponse(network_start_response) => {
                                if network_start_response.status == EAdpStatus::G3_SUCCESS {
                                    
                                    log::info!("received network start response, starting interface for address ");
                                    if self.tun_tx.is_some() {
                                        log::warn!(
                                            "Received network start for device already started"
                                        );
                                    } else {
                                        let request = request::AdpGetRequest::new (adp::EAdpPibAttribute::ADP_IB_COORD_SHORT_ADDRESS, 0);
                                        match self.cmd_tx.send(usi::Message::UsiOut(request.into())) {
                                            Ok(_) => {log::info!("Send to usi ")},
                                            Err(e) => {log::warn!("Failed to send to usi {}", e)},
                                        }
                                    }
                                }
                                else{
                                    log::error!("Failed to start network {:?}", network_start_response.status);
                                }
                            }
                            adp::Message::AdpG3NetworkJoinResponse(network_join_response) => {
                                if network_join_response.status == EAdpStatus::G3_SUCCESS {
                                    
                                    let short_addr = network_join_response.network_addr;
                                    if self.tun_tx.is_some() {
                                        log::warn!("Received network join response for address : {}, while device already starterd", short_addr);
                                    } else {
                                        let tun_device = TunDevice::new(tun_tx.clone());
                                        let (tx, mut rx) = flume::unbounded::<TunPayload>();
                                        self.tun_tx = Some(tx);
                                        tun_device.start(self.buffers_available.clone(),&settings, short_addr, rx, &extended_addr);
                                    }
                                }
                            }
                            adp::Message::AdpG3BufferEvent(event) => {
                                log::info!("Received buffer ready : {}", event.buffer_ready);
                                self.buffers_available.store(event.buffer_ready, Ordering::SeqCst);                                
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
                 if self.buffers_available.load(Ordering::SeqCst) {
                    match tun_rx.try_recv() {
                        Ok(msg) => {
                            match msg {
                                TunPayload::Data(pkt) => {

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
                                    //     }
                                    // }
                                    
                                    
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

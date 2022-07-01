use std::{
    collections::HashMap,
    intrinsics::transmute,
    io::Error,
    net::{Ipv4Addr, Ipv6Addr},
    sync::{atomic::AtomicBool, Arc},
    thread::{self, sleep_ms},
    vec,
};

use config::Config;
use pnet::packet::{
    ipv4::{Ipv4Flags, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    Packet,
};
use pnet_macros_support::packet;

use crate::app_config;

use std::sync::atomic::Ordering;

#[cfg(target_os = "macos")]
use tun::{self, AsyncDevice, Configuration, TunPacket, TunPacketCodec};

use crate::{
    adp::{self, EAdpStatus},
    request::AdpDataRequest,
    usi,
};
use pnet_datalink::Channel::Ethernet;

use rand::Rng;

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

    #[cfg(target_os = "linux")]
    pub fn start(self, short_addr: u16, mut rx: flume::Receiver<TunMessage>) {
        let interfaces = pnet_datalink::interfaces();
        let interface = interfaces
            .into_iter()
            .filter(|iface| iface.name == "tap0")
            .next()
            .unwrap();

        // Create a channel to receive on
        let (mut net_tx, mut net_rx) = match pnet_datalink::channel(&interface, Default::default())
        {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("rs_sender: unhandled channel type"),
            Err(e) => panic!("rs_sender: unable to create channel: {}", e),
        };

        thread::spawn(move || loop {
            match net_rx.next() {
                Ok(buf) => {
                    log::warn!("received tun packet {:?}", buf);
                    match self.listener.send(TunMessage::new(
                        self.short_addr,
                        TunPayload::Data(buf.to_vec()),
                    )) {
                        Ok(_) => {}
                        Err(e) => {
                            log::warn!("failed to send TunMessage to listener {}", e)
                        }
                    }
                }
                Err(_) => {
                    log::warn!("Failed to receive packet from tun interface")
                }
            }
        });

        thread::spawn(move || loop {
            match rx.recv() {
                Ok(tun_msg) => match tun_msg.get_payload() {
                    TunPayload::Data(buf) => {
                        net_tx.send_to(&buf, None);
                    }
                    TunPayload::Stop => {}
                    TunPayload::Error(_) => {}
                },
                Err(e) => log::warn!("Failed to receive from network manager : {}", e),
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
    pub fn ipv6_from_short_addr(pan_id: u16, short_addr: u16) -> Ipv6Addr {
        Ipv6Addr::new(0xfe80, 0x0, 0x0, 0x0, pan_id, 0x00ff, 0xfe00, short_addr)
    }
    pub fn ipv6_addr_from_ipv4_addr(pan_id: u16, ipv4: &Ipv4Addr) -> Ipv6Addr {
        Self::ipv6_from_short_addr(pan_id, Self::short_addr_from_ipv4(ipv4))
    }
    pub fn ipv4_addr_from_ipv6(ipv6_addr: Ipv6Addr) -> Ipv4Addr {
        let (pan_id, short_addr) = Self::pan_id_and_short_addr_from_ipv6(&ipv6_addr);
        Self::ipv4_from_short_addr(short_addr)
    }
    pub fn pan_id_and_short_addr_from_ipv6(ipv6: &Ipv6Addr) -> (u16, u16) {
        log::trace!("---> pan_id_and_short_addr_from_ipv6 : {} ", ipv6);
        let segments = ipv6.segments();
        log::trace!("---> pan_id_and_short_addr_from_ipv6 : {:?} ", segments);
        (segments[4], segments[7])
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

    pub fn ipv6_from_ipv4(pan_id: u16, buf: &Vec<u8>) -> Option<Vec<u8>> {
        let ipv4_pkt = Ipv4Packet::new(&buf);
        if let Some(ipv4) = ipv4_pkt {
            let mut buf = [0u8; 1520];
            if let Some(ref mut ipv6) = MutableIpv6Packet::new(&mut buf) {
                ipv6.set_destination(Self::ipv6_addr_from_ipv4_addr(
                    pan_id,
                    &ipv4.get_destination(),
                ));
                ipv6.set_source(Self::ipv6_addr_from_ipv4_addr(pan_id, &ipv4.get_source()));
                ipv6.set_traffic_class(Self::dscp_ecn_to_traffic_class(
                    ipv4.get_dscp(),
                    ipv4.get_ecn(),
                ));
                ipv6.set_flow_label(0); //TODO
                ipv6.set_payload(ipv4.payload());
                ipv6.set_next_header(ipv4.get_next_level_protocol());
                ipv6.set_hop_limit(ipv4.get_ttl());
                ipv6.set_version(6);
                ipv6.set_payload_length(ipv4.payload().len().try_into().unwrap());

                return Some(ipv6.packet()[..].to_vec());
            }
        }
        None
    }

    pub fn ipv4_from_ipv6(buf: &mut Vec<u8>) -> Option<(Vec<u8>, u16, u16)> {
        log::trace!("-->ipv4_from_ipv6 : {:?}", buf);
        let mut ipv6_pkt = Ipv6Packet::new(buf)?;
        let dst = Self::ipv4_addr_from_ipv6(ipv6_pkt.get_destination());
        let src = Self::ipv4_addr_from_ipv6(ipv6_pkt.get_source());
        let (dscp, ecn) = Self::traffic_class_to_dscp_ecn(ipv6_pkt.get_traffic_class());

        let mut bytes = vec![0xa5; 1520];
        let mut packet = MutableIpv4Packet::new(&mut bytes)?;

        packet.set_version(4);

        packet.set_dscp(dscp);
        packet.set_ecn(ecn);
        packet.set_header_length(20);
        packet.set_total_length((20 + ipv6_pkt.payload().len()).try_into().unwrap());
        packet.set_identification(0x42);
        packet.set_flags(Ipv4Flags::DontFragment);
        packet.set_fragment_offset(0);
        packet.set_next_level_protocol(ipv6_pkt.get_next_header());
        packet.set_ttl(ipv6_pkt.get_hop_limit());
        let (pan_id, short_addr) =
            Self::pan_id_and_short_addr_from_ipv6(&ipv6_pkt.get_destination());
        Some((packet.packet()[..].to_vec(), pan_id, short_addr))
        // match ipv6_pkt.next_header() {

        //     IpProtocol::Udp => {
        //         let mut udp_pkt = UdpPacket::new_checked(ipv6_pkt.payload_mut())?;
        //         udp_pkt.set_checksum(0);
        //         packet.payload_mut().copy_from_slice(udp_pkt.into_inner());
        //         packet.set_checksum(0);
        //     },
        //     _ => {
        //         packet.payload_mut().copy_from_slice(ipv6_pkt.payload_mut());
        //     }
        // }

        // let (pan_id, short_addr) =
        //     Self::pan_id_and_short_addr_from_ipv6(&ipv6_pkt.dst_addr().into());
        // let payload_len = packet.total_len().clone() as usize;
        // Ok((
        //     ipv6_pkt.next_header(),
        //     packet.into_inner()[..payload_len].to_vec(),
        //     pan_id,
        //     short_addr,
        // ))
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

    // pub fn config_from_short_addr (short_addr: u16) -> Configuration {
    //     let ipv4 = Self::ipv4_from_short_addr(short_addr);
    //     let mut config = tun::Configuration::default();

    //     config.address(&ipv4).netmask((255, 255, 0, 0)).up();
    //     #[cfg(target_os = "linux")]
    //     config.platform(|config| {
    //         config.packet_information(true);
    //     });
    //     config
    // }

    // fn start_tun(&mut self, short_addr: u16) -> Option<(posix::Reader, posix::Writer)> {
    //     let ipv4 = Self::ipv4_from_short_addr(short_addr);
    //     let mut config = tun::Configuration::default();

    //     config.address(&ipv4).netmask((255, 255, 0, 0)).up();
    //     tun::create(&config).map_or(None, |v| Some(v.split()))
    // }

    pub fn start(mut self, mut rx: flume::Receiver<adp::Message>) {
        let (tun_tx, mut tun_rx) = flume::unbounded::<TunMessage>();
        log::trace!("network manager starting ...");

        thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        log::trace!("Network manager received message : {:?}", msg);
                        match msg {
                            adp::Message::AdpG3DataEvent(g3_data) => {
                                match Self::ipv4_from_ipv6(&mut g3_data.nsdu.clone()) {
                                    Some((pkt, pan_id, short_addr_dst)) => {
                                        if let Some(tx) = self.tun_devices.get(&short_addr_dst) {
                                            log::trace!("found sender for short addr {} -- sending TunPayload::Data", short_addr_dst);

                                            match tx.send(TunMessage {
                                                short_addr: short_addr_dst,
                                                payload: TunPayload::Data(pkt),
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
                                    _ => {
                                        log::warn!("Failed to transform message to ipv4");
                                    }
                                }
                            }
                            adp::Message::AdpG3NetworkStartResponse(network_start_response) => {
                                if network_start_response.status == EAdpStatus::G3_SUCCESS {
                                    let short_addr = 0u16; //TODO, get the actual address from configuration
                                    log::trace!("received network start response, starting interface for address {}", short_addr);
                                    if self.tun_devices.contains_key(&short_addr) {
                                        log::warn!(
                                            "Received network start for device already started"
                                        );
                                    } else {
                                        let tun_device = TunDevice::new(short_addr, tun_tx.clone());
                                        let (tx, mut rx) = flume::unbounded::<TunMessage>();
                                        self.tun_devices.insert(short_addr, tx);
                                        tun_device.start(short_addr, rx);
                                    }
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

                                        tun_device.start(short_addr, rx);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {}
                }
                match tun_rx.try_recv() {
                    Ok(msg) => {
                        match msg.payload {
                            TunPayload::Data(pkt) => {
                                match Self::ipv6_from_ipv4(self.pan_id, &pkt) {
                                    Some(pkt) => {
                                        log::trace!("Sending ipv6 packet to G3 {:?}", pkt);
                                        let data_request = AdpDataRequest::new(
                                            rand::thread_rng().gen(),
                                            &pkt,
                                            true,
                                            0,
                                        );
                                        // sleep(Duration::from_millis(100)).await;
                                        self.cmd_tx.send(usi::Message::UsiOut(data_request.into()));
                                    }
                                    _ => {
                                        log::warn!("Failed to convert ipv6 packet to ipv4 packet");
                                    }
                                }
                            }
                            TunPayload::Stop => { //Should we use this as a notification that the device is stopped or should we have a separate message
                            }
                            TunPayload::Error(e) => {
                                log::trace!("Received error from device");
                            }
                        }
                    }
                    Err(_) => {}
                }
                sleep_ms(10); //TODO, spin threads and recv instead of try_recv
            }
        });
    }
}

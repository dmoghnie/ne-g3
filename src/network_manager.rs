use std::{
    collections::HashMap,
    intrinsics::transmute,
    io::Error,
    net::{Ipv4Addr, Ipv6Addr},
    sync::{atomic::AtomicBool, Arc},
    thread::{self, sleep_ms}, vec,
};

use bytes::{Buf, BytesMut};

use config::Config;

use futures::{future, stream::FuturesUnordered, StreamExt};
use packet::buffer::Buffer;

use crate::app_config;
use smoltcp::socket::{tcp, udp};
use smoltcp::wire::{self, IpProtocol, Ipv4Packet, Ipv6Packet, UdpPacket};
use std::sync::atomic::Ordering;

#[cfg(target_os = "macos")]
use tun::{self, AsyncDevice, Configuration, TunPacket, TunPacketCodec};

use crate::{
    adp::{self, EAdpStatus},
    request::AdpDataRequest,
    usi,
};

use rand::Rng;

#[derive(Debug)]
enum TunPayload {
    Udp(Vec<u8>),
    Tcp(Vec<u8>),
    Icmp(Vec<u8>),
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
        use std::{collections::BTreeMap, os::unix::prelude::AsRawFd};

        use serialport::new;
        use smoltcp::phy::{wait as phy_wait, RawSocket};
        use smoltcp::socket::{AnySocket, icmp};
        use smoltcp::time::Duration;
        use smoltcp::{
            iface::{FragmentsCache, InterfaceBuilder, SocketSet},
            phy::{Medium, TunTapInterface},
            socket::raw,
            time::Instant,
            wire::{IpAddress, IpCidr, IpProtocol, IpVersion, Ipv4Packet},
        };

        let ipv4_addr = NetworkManager::ipv4_from_short_addr(short_addr);

        thread::spawn(move || {
            let mut device = TunTapInterface::new(&app_config::TUN_NAME, Medium::Ip).unwrap();
            let fd = device.as_raw_fd();

            let ip_addr = IpCidr::new(IpAddress::from(ipv4_addr), 16);

            let mut iface = InterfaceBuilder::new()
                .ip_addrs([ip_addr])
                // .sixlowpan_fragments_cache(FragmentsCache::new(vec![], BTreeMap::new()))
                .ipv4_fragments_cache(FragmentsCache::new(vec![], BTreeMap::new()))
                .finalize(&mut device);

            let mut sockets = SocketSet::new(vec![]);

            let tcp_raw_rx_buffer =
                raw::PacketBuffer::new(vec![raw::PacketMetadata::EMPTY; 2], vec![0; 2560]);
            let tcp_raw_tx_buffer =
                raw::PacketBuffer::new(vec![raw::PacketMetadata::EMPTY; 2], vec![0; 2560]);
            let tcp_raw_socket = raw::Socket::new(
                IpVersion::Ipv4,
                IpProtocol::Tcp,
                tcp_raw_rx_buffer,
                tcp_raw_tx_buffer,
            );
            let tcp_raw_handle = sockets.add(tcp_raw_socket);

            let udp_raw_rx_buffer =
                raw::PacketBuffer::new(vec![raw::PacketMetadata::EMPTY; 2], vec![0; 2560]);
            let udp_raw_tx_buffer =
                raw::PacketBuffer::new(vec![raw::PacketMetadata::EMPTY; 2], vec![0; 2560]);
            let udp_raw_socket = raw::Socket::new(
                IpVersion::Ipv4,
                IpProtocol::Udp,
                udp_raw_rx_buffer,
                udp_raw_tx_buffer,
            );
            let udp_raw_handle = sockets.add(udp_raw_socket);

            let icmp_raw_rx_buffer =
            raw::PacketBuffer::new(vec![raw::PacketMetadata::EMPTY; 2], vec![0; 2560]);
            let icmp_raw_tx_buffer =
            raw::PacketBuffer::new(vec![raw::PacketMetadata::EMPTY; 2], vec![0; 2560]);
            let icmp_raw_socket = raw::Socket::new(
            IpVersion::Ipv4,
            IpProtocol::Icmp,
            icmp_raw_rx_buffer,
            icmp_raw_tx_buffer,
            );
            let icmp_raw_handle = sockets.add(icmp_raw_socket);

            loop {
                let timestamp = Instant::now();
                match iface.poll(timestamp, &mut device, &mut sockets) {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("poll error: {}", e);
                    }
                }

                for (handle, s) in sockets.iter_mut() {
                    if let Some(s) = raw::Socket::downcast_mut(s){
                        let protocol = s.ip_protocol().clone();
                        
                        if s.can_recv() {
                            log::trace!("socket {} : {} can received", s.ip_version(), s.ip_protocol());
                            match s.recv() {
                                Ok(buf) => match protocol {
                                    IpProtocol::HopByHop => {}
                                    IpProtocol::Icmp => {
                                        match self.listener.send(TunMessage::new(
                                            self.short_addr,
                                            TunPayload::Icmp(buf.to_vec()),
                                        )) {
                                            Ok(_) => {},
                                            Err(e) => {log::warn!("failed to send TunMessage to listener {}", e)},
                                        }
                                    },
                                    IpProtocol::Igmp => {}
                                    IpProtocol::Tcp => {
                                        match self.listener.send(TunMessage::new(
                                            self.short_addr,
                                            TunPayload::Tcp(buf.to_vec()),
                                        )) {
                                            Ok(_) => {},
                                            Err(e) => {log::warn!("failed to send TunMessage to listener {}", e)},
                                        }
                                    }
                                    IpProtocol::Udp => {
                                        match self.listener.send(TunMessage::new(
                                            self.short_addr,
                                            TunPayload::Udp(buf.to_vec()),
                                        )){
                                            Ok(_) => {},
                                            Err(e) => {log::warn!("failed to send TunMessage to listener {}", e)},
                                        }
                                    }
                                    IpProtocol::Ipv6Route => {}
                                    IpProtocol::Ipv6Frag => {}
                                    IpProtocol::Icmpv6 => {}
                                    IpProtocol::Ipv6NoNxt => {}
                                    IpProtocol::Ipv6Opts => {}
                                    IpProtocol::Unknown(_) => {}
                                },
                                Err(e) => {
                                    log::warn!("Failed to receive data from socket : {:?}", e)
                                }
                            }
                        }
                    }
                    
                }


                match rx.try_recv() {
                    Ok(tun_msg) => {
                        match tun_msg.get_payload() {
                            TunPayload::Udp(data) => {
                                log::trace!("TUN interface sending UDP {:?}", data);
                                let udp_socket = sockets.get_mut::<raw::Socket>(udp_raw_handle);
                                match udp_socket.send_slice(&data) {
                                    Ok(_) => log::trace!("TUN interface sent UDP data"),
                                    Err(e) => {
                                        log::warn!("TUN interface failed ot send UDP {:?}", e);
                                    }
                                }
                            }
                            TunPayload::Tcp(data) => {
                                log::trace!("TUN interface sending TCP {:?}", data);
                                let tcp_socket = sockets.get_mut::<raw::Socket>(tcp_raw_handle);
                                match tcp_socket.send_slice(&data) {
                                    Ok(_) => log::trace!("TUN interface sent TCP data"),
                                    Err(e) => {
                                        log::warn!("TUN interface failed ot send TCP {:?}", e);
                                    }
                                }
                            }
                            TunPayload::Icmp(data) => {
                                log::trace!("TUN interface sending ICMP {:?}", data);
                                let icmp_socket = sockets.get_mut::<raw::Socket>(icmp_raw_handle);
                                match icmp_socket.send_slice(&data) {
                                    Ok(_) => log::trace!("TUN interface sent ICMP data"),
                                    Err(e) => {
                                        log::warn!("TUN interface failed ot send ICMP {:?}", e);
                                    }
                                }
                            }
                            TunPayload::Stop => {
                                log::warn!("Stop not implmented yet");
                            }
                            TunPayload::Error(e) => {
                                log::warn!("TunPayload error ")
                            }
                        }
                        // socket.send_slice(tun_msg.get_payload())
                    }
                    Err(e) => {}
                }

                phy_wait(fd, Some(Duration::from_millis(100))).expect("wait error");
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

    pub fn ipv6_from_ipv4(pan_id: u16, buf: &Vec<u8>) -> Result<Vec<u8>, wire::Error> {
        let ipv4_pkt = Ipv4Packet::new_checked(buf)?;

        let dst = Self::ipv6_addr_from_ipv4_addr(pan_id, &ipv4_pkt.dst_addr().into());
        let src = Self::ipv6_addr_from_ipv4_addr(pan_id, &ipv4_pkt.src_addr().into());
        let traffic_class = Self::dscp_ecn_to_traffic_class(ipv4_pkt.dscp(), ipv4_pkt.ecn());

        let mut bytes = vec![0xff; 1280];
        let mut packet = Ipv6Packet::new_unchecked(&mut bytes);
        // Version, Traffic Class, and Flow Label are not
        // byte aligned. make sure the setters and getters
        // do not interfere with each other.
        packet.set_version(6);
        packet.set_traffic_class(traffic_class);
        packet.set_flow_label(0); //TODO
        
        packet.set_payload_len(ipv4_pkt.payload().len().try_into().unwrap());
        packet.set_next_header(ipv4_pkt.next_header());
        packet.set_hop_limit(ipv4_pkt.hop_limit());
        packet.set_src_addr(src.into());
        packet.set_dst_addr(dst.into());
        packet.payload_mut().copy_from_slice(ipv4_pkt.payload());

        let payload_len = packet.total_len().clone();
        Ok(packet.into_inner()[..payload_len].to_vec())
    }

    pub fn ipv4_from_ipv6(buf: &mut Vec<u8>) -> Result<(IpProtocol, Vec<u8>, u16, u16), wire::Error> {
        log::trace!("-->ipv4_from_ipv6 : {:?}", buf);
        let mut ipv6_pkt = Ipv6Packet::new_checked(buf)?;
        let dst = Self::ipv4_addr_from_ipv6(ipv6_pkt.dst_addr().into());
        let src = Self::ipv4_addr_from_ipv6(ipv6_pkt.src_addr().into());
        let (dscp, ecn) = Self::traffic_class_to_dscp_ecn(ipv6_pkt.traffic_class());

        let mut bytes = vec![0xa5; 1280];
        let mut packet = Ipv4Packet::new_unchecked(&mut bytes);
        
        packet.set_version(4);
        packet.set_header_len(20);
        packet.set_dscp(dscp);
        packet.set_ecn(ecn);
        packet.set_total_len(20 + ipv6_pkt.payload_len());
        packet.set_ident(0);
        packet.clear_flags();
        packet.set_more_frags(false);
        packet.set_dont_frag(true);
        packet.set_frag_offset(0);
        packet.set_hop_limit(ipv6_pkt.hop_limit());
        packet.set_next_header(ipv6_pkt.next_header());
        packet.set_src_addr(src.into());
        packet.set_dst_addr(dst.into());
        // packet.fill_checksum();

        match ipv6_pkt.next_header() {

            IpProtocol::Udp => {
                let mut udp_pkt = UdpPacket::new_checked(ipv6_pkt.payload_mut())?;
                udp_pkt.set_checksum(0);
                packet.payload_mut().copy_from_slice(udp_pkt.into_inner());
                packet.set_checksum(0);
            },
            _ => {
                packet.payload_mut().copy_from_slice(ipv6_pkt.payload_mut());
            }
        }

        
        let (pan_id, short_addr) =
            Self::pan_id_and_short_addr_from_ipv6(&ipv6_pkt.dst_addr().into());
        let payload_len = packet.total_len().clone() as usize;
        Ok((
            ipv6_pkt.next_header(),
            packet.into_inner()[..payload_len].to_vec(),
            pan_id,
            short_addr,
        ))
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
                                    Ok((protocol, pkt, pan_id, short_addr_dst)) => {
                                        if let Some(tx) = self.tun_devices.get(&short_addr_dst) {
                                            log::trace!("found sender for short addr {} -- sending TunPayload::Data", short_addr_dst);
                                            let payload = match protocol {
                                                IpProtocol::Tcp => Some(TunPayload::Tcp(pkt)),
                                                IpProtocol::Udp => Some(TunPayload::Udp(pkt)),
                                                IpProtocol::Icmp => Some(TunPayload::Icmp(pkt)),
                                                _ => {
                                                    log::warn!("ipv4_from_ipv6 protocol not implemented {}", protocol);
                                                    None
                                                }
                                            };
                                            if let Some(payload) = payload {
                                                match tx.send(TunMessage {
                                                    short_addr: short_addr_dst,
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
                                    Err(_) => {
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
                            TunPayload::Udp(pkt) | TunPayload::Tcp(pkt) | TunPayload::Icmp(pkt) => {
                                match Self::ipv6_from_ipv4(self.pan_id, &pkt) {
                                    Ok(pkt) => {
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
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to convert ipv6 packet to ipv4 packet : {}",
                                            e
                                        );
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
                sleep_ms(100);//TODO, spin threads and recv instead of try_recv
            }
        });
    }
}

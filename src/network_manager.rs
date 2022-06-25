use std::{
    collections::HashMap,
    intrinsics::transmute,
    net::{Ipv4Addr, Ipv6Addr},
    sync::{atomic::AtomicBool, Arc},
    thread, vec, io::Error,
};

use bytes::{BytesMut, Buf};

use config::Config;

use packet::{icmp, ip::{self, Protocol}, ether, buffer, Builder, AsPacket, Packet, udp, tcp};
use packet::buffer::Buffer;
use futures::{stream::FuturesUnordered, StreamExt, future};

use futures::{SinkExt, channel::mpsc::UnboundedReceiver};
use tokio::{time::{self, Duration}, pin, sync::mpsc, task::JoinHandle, io::AsyncReadExt, io::AsyncWriteExt};
use tokio_util::codec::{Decoder, FramedRead};
use tokio::io;

use std::sync::atomic::Ordering;

#[cfg(target_os = "macos")]
use tun::{self, AsyncDevice, Configuration, TunPacket, TunPacketCodec};

use crate::{
    adp::{self, EAdpStatus},
    usi, request::AdpDataRequest,
};

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
    listener: mpsc::UnboundedSender<TunMessage>,

}

impl TunDevice {
    pub fn new(short_addr: u16, listener: mpsc::UnboundedSender<TunMessage>) -> Self {
        TunDevice {
            short_addr,
            listener,

        }
    }

    #[cfg(target_os = "linux")]
    pub fn start_async(
        self,
        short_addr: u16, mut rx: tokio::sync::mpsc::UnboundedReceiver<TunMessage>
    ) {
        
        let ipv4_addr = NetworkManager::ipv4_from_short_addr(short_addr);
        let dev =  tokio_tun::TunBuilder::new()
        .name("")
        .address(ipv4_addr)
        .netmask("255.255.0.0".parse().unwrap())            // if name is empty, then it is set by kernel.
        .tap(false)
        .mtu(1280)          // false (default): TUN, true: TAP.
        .packet_info(false)  // false: IFF_NO_PI, default is true.
        .up()                // or set it up manually using `sudo ip link set <tun-name> up`.
        .try_build().unwrap(); //TODO
        
        let (mut reader, mut writer) = tokio::io::split(dev);
        
        

        // let mut running = AtomicBool::new(true);
        let running = Arc::new(AtomicBool::new(true));
        let should_run = running.clone();
        let f1 = tokio::task::spawn(async move {
            log::trace!("start_async read next ....");
            let mut buf = [0u8; 2048];
            loop {                
                match tokio::time::timeout(Duration::from_millis(5000), reader.read(&mut buf)).await {
                    Ok(r_size) => {
                        log::trace!("Tun reader received packet {:?}", r_size);
                        if let Ok(size) = r_size {  
                            let pkt = ip::v4::Packet::new(&buf[..size]);                          
                            match pkt {
                                Ok(packet) => {
                                    log::trace!("ipv4 : {:?}", packet);
                                    self.listener.send(TunMessage::new(
                                        self.short_addr,
                                        TunPayload::Data(packet.as_ref().to_vec()),
                                    )); //TODO check the result
                                }
                                Err(e) => {
                                    log::warn!("TunDevice error reading {}", e);
                                    self.listener.send(TunMessage::new(
                                        self.short_addr,
                                        TunPayload::Error(()),
                                    ));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::trace!("tun device read timeout");
                        if !running.load(Ordering::SeqCst) {
                            log::trace!("Breaking from tun reader");
                            break;
                        }
                    }
                }
            }
        });
        // .await;
        // match result {
        //     Ok(_) => {},
        //     Err(e) => {
        //         log::warn!("Failed to await {}", e);
        //     },
        // }
        
        let f2 = tokio::task::spawn(async move {
            log::trace!("spawning packet writer");
            loop {
                match rx.recv().await {
                    Some(msg) => {
                        log::trace!("Tun writer, writing packet : {:?}", msg);
                        match msg.get_payload() {
                            TunPayload::Data(packet) => {
                                
                                match writer.write(&packet).await {
                                    Ok(_) => log::trace!("ipv4 msg sent"),
                                    Err(e) => log::warn!("Failed to write msg : {:?}", e),
                                }
                            }
                            TunPayload::Stop => {
                                //Remove from device list ??
                                should_run.store(false, Ordering::SeqCst);
                                break;
                            }
                            TunPayload::Error(_) => {} //TODO check error
                        }
                    }
                    None => {
                        log::warn!("writer : Failed to receive network message");
                    },                    
                }
            }
        });
        // match r {
        //     Ok(_) => {},
        //     Err(e) => {log::warn!("Failed to start packet writer : {}", e)},
        // }
        // .await;
        log::trace!("start async end ...");

    }

    #[cfg(target_os = "macos")]
    pub fn start_async(
        self,
        short_addr: u16, mut rx: tokio::sync::mpsc::UnboundedReceiver<TunMessage>
    ) {
        let ipv4 = NetworkManager::ipv4_from_short_addr(short_addr);
        let mut config = tun::Configuration::default();

        config.mtu(1280).address(&ipv4).netmask((255, 255, 0, 0)).up();

        let dev = tun::create_as_async(&config).unwrap();
        
        let (mut writer, mut reader) = dev.into_framed().split();
        log::trace!("Start async ... for {:?}", config);
        // let mut running = AtomicBool::new(true);
        let running = Arc::new(AtomicBool::new(true));
        let should_run = running.clone();
        let f1 = tokio::task::spawn(async move {
            log::trace!("start_async read next ....");
            loop {                
                match tokio::time::timeout(Duration::from_millis(5000), reader.next()).await {
                    Ok(packet) => {
                        log::trace!("Tun reader received packet {:?}", packet);
                        if let Some(pkt) = packet {                            
                            match pkt {
                                Ok(packet) => {
                                    log::trace!("ipv4 : {:?}", packet.get_bytes());
                                    self.listener.send(TunMessage::new(
                                        self.short_addr,
                                        TunPayload::Data(packet.get_bytes().to_vec()),
                                    )); //TODO check the result
                                }
                                Err(e) => {
                                    log::warn!("TunDevice error reading {}", e);
                                    self.listener.send(TunMessage::new(
                                        self.short_addr,
                                        TunPayload::Error(()),
                                    ));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::trace!("tun device read timeout");
                        if !running.load(Ordering::SeqCst) {
                            log::trace!("Breaking from tun reader");
                            break;
                        }
                    }
                }
            }
        });
        // .await;
        // match result {
        //     Ok(_) => {},
        //     Err(e) => {
        //         log::warn!("Failed to await {}", e);
        //     },
        // }
        
        let f2 = tokio::task::spawn(async move {
            log::trace!("spawning packet writer");
            loop {
                match rx.recv().await {
                    Some(msg) => {
                        log::trace!("Tun writer, writing packet : {:?}", msg);
                        match msg.get_payload() {
                            TunPayload::Data(packet) => {
                                
                                match writer.send(TunPacket::new(packet)).await {
                                    Ok(_) => log::trace!("ipv4 msg sent"),
                                    Err(e) => log::warn!("Failed to write msg : {:?}", e),
                                }
                            }
                            TunPayload::Stop => {
                                //Remove from device list ??
                                should_run.store(false, Ordering::SeqCst);
                                break;
                            }
                            TunPayload::Error(_) => {} //TODO check error
                        }
                    }
                    None => {
                        log::warn!("writer : Failed to receive network message");
                    },                    
                }
            }
        });
        // match r {
        //     Ok(_) => {},
        //     Err(e) => {log::warn!("Failed to start packet writer : {}", e)},
        // }
        // .await;
        log::trace!("start async end ...");

    }
}

pub struct NetworkManager {
    pan_id: u16,
    cmd_tx: flume::Sender<usi::Message>,
    tun_devices: HashMap<u16, mpsc::UnboundedSender::<TunMessage>>,
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
        Ipv4Addr::new(10u8, 0u8, b[0], b[1]) //TODO parameterize
    }
    pub fn short_addr_from_ipv4 (ipv4: &Ipv4Addr) -> u16 {
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
    pub fn dscp_ecn_to_traffic_class (dscp: u8, ecn: u8) -> u8 {
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
    
    pub fn ipv6_from_ipv4 (pan_id: u16, ipv4_pkt: ip::v4::Packet<Vec<u8>>) -> Result<Vec<u8>, packet::Error> {
        let dst = Self::ipv6_addr_from_ipv4_addr(pan_id,&ipv4_pkt.destination());
        let src = Self::ipv6_addr_from_ipv4_addr(pan_id, &ipv4_pkt.source());
        
        let v = ip::v6::Builder::default()
            .traffic_class(Self::dscp_ecn_to_traffic_class(ipv4_pkt.dscp(), ipv4_pkt.ecn()))?
            .flow_label(0)?//TODO, 
            .payload_length(ipv4_pkt.payload().len() as u16)?
            .next_header(ipv4_pkt.protocol().into())?
            .hop_limit(ipv4_pkt.ttl())?
            .source(src)?
            .destination(dst)?
            .payload(ipv4_pkt.payload())?
            .build()?;


        Ok(v)

    }
    pub fn ipv4_from_ipv6 (ipv6_pkt: ip::v6::Packet<Vec<u8>>) -> Result<Vec<u8>, packet::Error> {
        log::trace!("-->ipv4_from_ipv6 : {:?}", ipv6_pkt);
        let dst = Self::ipv4_addr_from_ipv6(ipv6_pkt.destination());
        let src = Self::ipv4_addr_from_ipv6(ipv6_pkt.source());
        let (dscp, ecn) = Self::traffic_class_to_dscp_ecn(ipv6_pkt.traffic_class());
    
        let protocol: ip::Protocol = ipv6_pkt.next_header().into();
        match protocol {
            Protocol::Udp => {
                let udp = udp::Packet::new(ipv6_pkt.payload());
                log::trace!("ipv4_from_ipv6 : udp - {:?}", udp);
                if let Ok(udp) = udp {
                    let src_port = udp.source();
                    let dst_port = udp.destination();
                    
                    let v = ip::v4::Builder::default().id(0x42)?.dscp(dscp)?.ecn(ecn)?
                        .source(src)?.destination(dst)?
                        .ttl(ipv6_pkt.hop_limit())?.udp()?.source(src_port)?.destination(dst_port)?.payload(udp.payload())?.build();
                    return v;
                }
            },
            Protocol::Tcp => {
                log::trace!("-->ipv4_from_ipv6 : tcp --");
                log::trace!("-->ipv4_from_ipv6 : ipv6 payload : {:?}", ipv6_pkt.payload());
                let tcp = tcp::Packet::unchecked(ipv6_pkt.payload());
                log::trace!("-->ipv4_from_ipv6 : tcp {:?}", tcp);

                let v = ip::v4::Builder::default().id(0x42)?.dscp(dscp)?.ecn(ecn)?
                .source(src)?.destination(dst)?
                .ttl(ipv6_pkt.hop_limit())?
                .tcp()?.source(tcp.source())?.destination(tcp.destination())?
                .sequence(tcp.sequence())?.acknowledgment(tcp.acknowledgment())?
                .window(1280)?.pointer(tcp.pointer())?.flags(tcp.flags())?.payload(tcp.payload())?.build();
                if let Ok(pkt_data) = &v {
                    log::trace!("-->ipv4_from_ipv6 : result : {:?}", ip::v4::Packet::new(pkt_data));
                }
                return v;
                
            },
            Protocol::Icmp => {
                // let icmp = icmp::Packet::unchecked(ipv6_pkt.payload());
                // let v = ip::v4::Builder::default().id(0x42)?.dscp(dscp)?.ecn(ecn)?
                // .source(src)?.destination(dst)?
                // .ttl(ipv6_pkt.hop_limit())?
                // .icmp()?.
            },
            _ => {
                log::warn!("Received unsupported protocol {:?}", protocol);
                return Err(packet::Error::InvalidPacket);
            }
        }
        
        // Ok(udp?.as_ref().to_vec())
        Err(packet::Error::InvalidPacket)
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

    pub async fn start(mut self, mut rx: flume::Receiver<adp::Message>) {
        log::trace!("network manager starting ...");
        let mut futures:FuturesUnordered<JoinHandle<()>> = FuturesUnordered::new();

        let (tun_tx, mut tun_rx) = mpsc::unbounded_channel::<TunMessage>();
        
        let f1 = tokio::spawn(async move {
            log::trace!("network manager starting adp receiver ...");
            loop {
                match rx.recv_async().await {
                    Ok(msg) => {
                        log::trace!("NetworkManager received message : {:?}", msg);
                        match msg {
                            adp::Message::AdpG3DataEvent(g3_data) => {                                
                                match ip::v6::Packet::new(g3_data.nsdu) {
                                    Ok(pkt) => {
                                        log::trace!("Received ipv6 packet from G3 {:?} - payload : {:?}", pkt, pkt.payload()); 
                                        let (_, short_addr_dst) = Self::pan_id_and_short_addr_from_ipv6(&pkt.destination());   
                                        
                                        match Self::ipv4_from_ipv6(pkt) {
                                            Ok(ipv4_pkt) => {                                                                                               
                                                log::trace!("sending ipv4 packet : {:?} -> dst short address {}", ipv4_pkt, short_addr_dst); 
                                                if let Some(tx) = self.tun_devices.get(&short_addr_dst) {
                                                    log::trace!("found sender for short addr {} -- sending TunPayload::Data", short_addr_dst);
                                                    match tx.send(TunMessage { short_addr: short_addr_dst, 
                                                        payload: TunPayload::Data(ipv4_pkt) }){
                                                            Ok(_) => {},
                                                            Err(e) => {log::warn!("Failed to send ipv4 packet to TUN : {}", e)},
                                                        }
                                                }
                                            },
                                            Err(e) => {log::warn!("Failed to transform ipv6 packet into ipv4")},
                                        }
                                    },
                                    Err(e) => {
                                        log::warn!("Failed to receive ipv6 packet from G3");
                                    },
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
                                        let (tx, mut rx) = mpsc::unbounded_channel::<TunMessage>();                            
                                        self.tun_devices.insert(
                                            short_addr,
                                            tx
                                        );
                                        tun_device.start_async(short_addr, rx);
                                        
                                        
                                    }
                                }
                            }
                            adp::Message::AdpG3NetworkJoinResponse(network_join_response) => {
                                if network_join_response.status == EAdpStatus::G3_SUCCESS {
                                    let short_addr = network_join_response.network_addr;
                                    if self.tun_devices.contains_key(&network_join_response.network_addr) {
                                        log::warn!("Received network join response for address : {}, while device already in list", short_addr);        
                                    }
                                    else{                                        
                                        let tun_device = TunDevice::new(short_addr, tun_tx.clone());
                                        let (tx, mut rx) = mpsc::unbounded_channel::<TunMessage>();  
                                        self.tun_devices.insert(
                                            short_addr,
                                            tx
                                        );
                                       
                                        tun_device.start_async(short_addr, rx);
                                        
                                    }
                                    
                                }
                                
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        log::warn!("Network manager failed to receive message : {}", e);
                    }
                }
            }
        });

        let f2 = tokio::spawn(async move {
            log::trace!("Start Tun message processor ");
            loop {
                match tun_rx.recv().await {
                    Some(msg) => {
                        match msg.payload {
                            TunPayload::Data(pkt) => {
                                
                                match ip::Packet::new(pkt) {
                                    Ok(pkt) => {
                                        match pkt {
                                            ip::Packet::V4(ipv4_pkt) => {
                                                log::trace!("Received ipv4 packet from device ... {:?}", ipv4_pkt);
                                                match Self::ipv6_from_ipv4(self.pan_id, ipv4_pkt) {
                                                    Ok(ipv6_pkt) => {
                                                        log::trace!("Sending ipv6 packet to G3 {:?}", ipv6_pkt);
                                                        let data_request = AdpDataRequest::new(rand::thread_rng().gen(), &ipv6_pkt, true, 100);
                                                        self.cmd_tx.send(usi::Message::UsiOut(data_request.into()));
                                                    },
                                                    Err(e) => {
                                                        log::warn!("Failed to convert ipv6 packet to ipv4 packet : {}", e);
                                                    },
                                                }                                                
                                                
                                            },
                                            ip::Packet::V6(_) => {
                                                log::warn!("Received ipv6 on utun, not implemented yet");
                                            },
                                        }
                                    },
                                    Err(e) => {log::warn!("failed o transform tun message into packet : {}", e)},
                                }
                                
                                //TODO transform packet to ipv6 and send it to the G3 network
                            }
                            TunPayload::Stop => { //Should we use this as a notification that the device is stopped or should we have a separate message
                            }
                            TunPayload::Error(e) => {
                                log::trace!("Received error from device");
                            }
                        }
                    }
                    None => {
                        log::warn!("Failed to receive tun message from device");
                    }
                }
            }
        });
        futures.push(f1);
        futures.push(f2);
        while let Some(f) = futures.next().await {
            
        }
    }
}

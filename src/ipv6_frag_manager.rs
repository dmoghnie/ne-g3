

use pnet_packet::{
    ip::{IpNextHeaderProtocols::{self, Ipv6Frag, Hopopt, Ipv6Route, Ah, Esp, Ipv6Opts, Shim6, Test1, MobilityHeader, Hip, Test2}, IpNextHeaderProtocol},
    ipv6::{
        ExtensionIterable, ExtensionPacket, FragmentPacket, Ipv6Packet, MutableFragmentPacket,
        MutableIpv6Packet,
    },
    MutablePacket, Packet, PacketSize,
};
use rand::Rng;

pub fn get_fragment_offset<'a>(packet: &'a Ipv6Packet) -> Option<usize> {
    let mut itr = ExtensionIterable::new(&packet.payload());
    let mut offset = 0;
    for ext in itr {
        offset += ext.packet_size();
        if ext.get_next_header() == Ipv6Frag {
            return Some(offset + (packet.packet_size() - packet.get_payload_length() as usize));
        }
    }

    None
}

pub fn get_true_payload_offset<'a>(packet: &'a Ipv6Packet) -> usize {
    if ![0, 43, 44, 51, 50, 60, 135, 139, 140, 253, 254].contains(&packet.get_next_header().0) {
        return 40;
    }
    let itr = ExtensionIterable::new(&packet.payload());
    let mut offset = 0;
    for ext in itr {
        offset += ext.packet_size();
    }
    println!("offset : {}", offset);
    return offset + (packet.packet_size() - packet.get_payload_length() as usize);
}
pub fn get_true_payload<'a>(packet: &'a Ipv6Packet) -> IpNextHeaderProtocol {
    if ![0, 43, 44, 51, 50, 60, 135, 139, 140, 253, 254].contains(&packet.get_next_header().0) {
        return packet.get_next_header();
    }
    else{
        let next_headers = ExtensionIterable::new(&packet.payload())
            .map(|v| v.get_next_header()).collect::<Vec<_>>();
        *next_headers.last().unwrap_or(&IpNextHeaderProtocols::Test1)
    }
}

pub fn is_extension (protocol: IpNextHeaderProtocol) -> bool {
    protocol == Ipv6Route || protocol == Ipv6Frag || protocol == Ah || protocol == Esp || protocol == Ipv6Opts
        || protocol == MobilityHeader || protocol == Hip || protocol == Shim6 || protocol == Test1|| protocol == Test2
}

pub fn fragment_packet(packet: Ipv6Packet, max_size: usize) -> Vec<Vec<u8>> {
    let mut result:Vec<Vec<u8>> = Vec::new();

    if packet.packet().len() < max_size {
        result.push(packet.packet()[..].to_vec());
    } else {
        if get_fragment_offset(&packet).is_some() {
            println!("Cannot fragment packet that is already a fragment. not implemented");
        } else {
            let fixed_size = get_true_payload_offset(&packet);
            let mut payload_size = packet.packet().len() - fixed_size;
            let available_payload_size =
                ((max_size - (fixed_size + 8/*fragment header */)) / 8usize) * 8usize; //align to 8

            let mut rng = rand::thread_rng();
            let mut frag_offset = 0usize;
            let id: u32 = rng.gen();
            while payload_size > 0 {
                let mut buf = [0u8; 2048];
                buf[..40].copy_from_slice(&packet.packet()[..40]);
                
                    let mut f_buffer = [0u8; 8];
                    let mut fragment = MutableFragmentPacket::new(&mut f_buffer).unwrap();
                    fragment.set_id(id);
                    fragment.set_next_header(packet.get_next_header());
                    fragment.set_fragment_offset(frag_offset as u16);
                    
                    if payload_size > available_payload_size {   
                        fragment.set_last_fragment(false);
                        buf[40..48].copy_from_slice(&fragment.packet()[..]);     
                        buf[48..fixed_size +8usize].copy_from_slice(&packet.packet()[40..fixed_size]);
                        buf[fixed_size +8usize .. fixed_size +8usize + available_payload_size]
                            .copy_from_slice(&packet.packet()[fixed_size + frag_offset .. fixed_size + available_payload_size + frag_offset]);
                        let mut new_packet = MutableIpv6Packet::new(&mut buf).unwrap();
                        new_packet.set_next_header(IpNextHeaderProtocols::Ipv6Frag);

                        new_packet.set_payload_length(available_payload_size as u16);
                        payload_size -= available_payload_size;
                        frag_offset += available_payload_size;                        
                        result.push(new_packet.packet()[..fixed_size+8+available_payload_size].to_vec());
                    }
                    else{
                        fragment.set_last_fragment(true);
                        buf[40..48].copy_from_slice(&fragment.packet()[..]);     
                        buf[48..fixed_size +8usize].copy_from_slice(&packet.packet()[40..fixed_size]);
                        buf[fixed_size +8usize .. fixed_size +8usize + payload_size]
                            .copy_from_slice(&packet.packet()[fixed_size + frag_offset .. fixed_size + payload_size + frag_offset]);
                        let mut new_packet = MutableIpv6Packet::new(&mut buf).unwrap();
                        new_packet.set_next_header(IpNextHeaderProtocols::Ipv6Frag);
                        
                        new_packet.set_payload_length(payload_size as u16);
                        result.push(new_packet.packet()[..fixed_size+8+payload_size].to_vec());
                        payload_size = 0;
                        
                    }
                    
                
            }

            println!("packet : {:?} fixed_size {}", packet, fixed_size);
        }
    }
    result
}



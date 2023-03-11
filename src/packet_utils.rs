pub(crate) mod PacketUtils {
    pub(crate) enum PacketProtocol {
        IPv4,
        IPv6,
        Other(u8),
    }

    pub(crate) fn infer_proto(buf: &[u8]) -> PacketProtocol {
        match buf[0] >> 4 {
            4 => PacketProtocol::IPv4,
            6 => PacketProtocol::IPv6,
            p => PacketProtocol::Other(p),
        }
    }
}
use crate::usi;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use usi::InMessage;
use std::fmt;

pub const G3_SERIAL_MSG_STATUS: u8 = 0;

/* COORDINATOR ACCESS */
pub const G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_BEGIN: u8 = 1;

pub const G3_SERIAL_MSG_COORD_INITIALIZE: u8 = (G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_BEGIN);
pub const G3_SERIAL_MSG_COORD_SET_REQUEST: u8 = (G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_BEGIN + 1);
pub const G3_SERIAL_MSG_COORD_GET_REQUEST: u8 = (G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_BEGIN + 2);
pub const G3_SERIAL_MSG_COORD_KICK_REQUEST: u8 = (G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_BEGIN + 3);
pub const G3_SERIAL_MSG_COORD_REKEYING_REQUEST: u8 =
    (G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_BEGIN + 4);

pub const G3_SERIAL_MSG_COORD_CONF_IND_MESSAGES_BEGIN: u8 =
    (G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_BEGIN + 5);
pub const G3_SERIAL_MSG_COORD_SET_CONFIRM: u8 = (G3_SERIAL_MSG_COORD_CONF_IND_MESSAGES_BEGIN);
pub const G3_SERIAL_MSG_COORD_GET_CONFIRM: u8 = (G3_SERIAL_MSG_COORD_CONF_IND_MESSAGES_BEGIN + 1);
pub const G3_SERIAL_MSG_COORD_JOIN_INDICATION: u8 =
    (G3_SERIAL_MSG_COORD_CONF_IND_MESSAGES_BEGIN + 2);
pub const G3_SERIAL_MSG_COORD_LEAVE_INDICATION: u8 =
    (G3_SERIAL_MSG_COORD_CONF_IND_MESSAGES_BEGIN + 3);

pub const G3_SERIAL_MSG_COORD_REQUEST_MESSAGES_END: u8 = (G3_SERIAL_MSG_COORD_LEAVE_INDICATION);

/* ADP ACCESS */
pub const G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN: u8 = 10;
pub const G3_SERIAL_MSG_ADP_INITIALIZE: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN);
pub const G3_SERIAL_MSG_ADP_DATA_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 1);
pub const G3_SERIAL_MSG_ADP_DISCOVERY_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 2);
pub const G3_SERIAL_MSG_ADP_NETWORK_START_REQUEST: u8 =
    (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 3);
pub const G3_SERIAL_MSG_ADP_NETWORK_JOIN_REQUEST: u8 =
    (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 4);
pub const G3_SERIAL_MSG_ADP_NETWORK_LEAVE_REQUEST: u8 =
    (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 5);
pub const G3_SERIAL_MSG_ADP_RESET_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 6);
pub const G3_SERIAL_MSG_ADP_SET_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 7);
pub const G3_SERIAL_MSG_ADP_GET_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 8);
pub const G3_SERIAL_MSG_ADP_LBP_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 9);
pub const G3_SERIAL_MSG_ADP_ROUTE_DISCOVERY_REQUEST: u8 =
    (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 10);
pub const G3_SERIAL_MSG_ADP_PATH_DISCOVERY_REQUEST: u8 =
    (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 11);
pub const G3_SERIAL_MSG_ADP_MAC_SET_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 12);
pub const G3_SERIAL_MSG_ADP_MAC_GET_REQUEST: u8 = (G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_BEGIN + 13);
pub const G3_SERIAL_MSG_ADP_REQUEST_MESSAGES_END: u8 = (G3_SERIAL_MSG_ADP_MAC_GET_REQUEST);

pub const G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN: u8 = 30;
pub const G3_SERIAL_MSG_ADP_DATA_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN);
pub const G3_SERIAL_MSG_ADP_DATA_INDICATION: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 1);
pub const G3_SERIAL_MSG_ADP_NETWORK_STATUS_INDICATION: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 2);
pub const G3_SERIAL_MSG_ADP_DISCOVERY_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 3);
pub const G3_SERIAL_MSG_ADP_NETWORK_START_CONFIRM: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 4);
pub const G3_SERIAL_MSG_ADP_NETWORK_JOIN_CONFIRM: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 5);
pub const G3_SERIAL_MSG_ADP_NETWORK_LEAVE_CONFIRM: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 6);
pub const G3_SERIAL_MSG_ADP_NETWORK_LEAVE_INDICATION: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 7);
pub const G3_SERIAL_MSG_ADP_RESET_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 8);
pub const G3_SERIAL_MSG_ADP_SET_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 9);
pub const G3_SERIAL_MSG_ADP_GET_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 10);
pub const G3_SERIAL_MSG_ADP_LBP_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 11);
pub const G3_SERIAL_MSG_ADP_LBP_INDICATION: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 12);
pub const G3_SERIAL_MSG_ADP_ROUTE_DISCOVERY_CONFIRM: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 13);
pub const G3_SERIAL_MSG_ADP_PATH_DISCOVERY_CONFIRM: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 14);
pub const G3_SERIAL_MSG_ADP_MAC_SET_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 15);
pub const G3_SERIAL_MSG_ADP_MAC_GET_CONFIRM: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 16);
pub const G3_SERIAL_MSG_ADP_BUFFER_INDICATION: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 17);
pub const G3_SERIAL_MSG_ADP_DISCOVERY_INDICATION: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 18);
pub const G3_SERIAL_MSG_ADP_PREQ_INDICATION: u8 = (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 19);
pub const G3_SERIAL_MSG_ADP_UPD_NON_VOLATILE_DATA_INDICATION: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 20);
pub const G3_SERIAL_MSG_ADP_ROUTE_NOT_FOUND_INDICATION: u8 =
    (G3_SERIAL_MSG_ADP_CONF_IND_MESSAGES_BEGIN + 21);

/* MAC ACCESS */
pub const G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN: u8 = 50;
pub const G3_SERIAL_MSG_MAC_INITIALIZE: u8 = (G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN);
pub const G3_SERIAL_MSG_MAC_DATA_REQUEST: u8 = (G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN + 1);
pub const G3_SERIAL_MSG_MAC_GET_REQUEST: u8 = (G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN + 2);
pub const G3_SERIAL_MSG_MAC_SET_REQUEST: u8 = (G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN + 3);
pub const G3_SERIAL_MSG_MAC_RESET_REQUEST: u8 = (G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN + 4);
pub const G3_SERIAL_MSG_MAC_SCAN_REQUEST: u8 = (G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN + 5);
pub const G3_SERIAL_MSG_MAC_START_REQUEST: u8 = (G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_BEGIN + 6);
pub const G3_SERIAL_MSG_MAC_REQUEST_MESSAGES_END: u8 = (G3_SERIAL_MSG_MAC_START_REQUEST);

pub const G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN: u8 = 60;
pub const G3_SERIAL_MSG_MAC_DATA_CONFIRM: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN);
pub const G3_SERIAL_MSG_MAC_DATA_INDICATION: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 1);
pub const G3_SERIAL_MSG_MAC_GET_CONFIRM: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 2);
pub const G3_SERIAL_MSG_MAC_SET_CONFIRM: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 3);
pub const G3_SERIAL_MSG_MAC_RESET_CONFIRM: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 4);
pub const G3_SERIAL_MSG_MAC_SCAN_CONFIRM: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 5);
pub const G3_SERIAL_MSG_MAC_BEACON_NOTIFY: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 6);
pub const G3_SERIAL_MSG_MAC_START_CONFIRM: u8 = (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 7);
pub const G3_SERIAL_MSG_MAC_COMM_STATUS_INDICATION: u8 =
    (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 8);
pub const G3_SERIAL_MSG_MAC_SNIFFER_INDICATION: u8 =
    (G3_SERIAL_MSG_MAC_CONF_IND_MESSAGES_BEGIN + 9);

pub const ADP_ADDRESS_16BITS: i32 = 2;
pub const ADP_ADDRESS_64BITS: i32 = 8;


pub enum EAdpMac_Modulation {
    MOD_ROBO = 0,
    MOD_BPSK,
    MOD_DBPSK,
    MOD_QPSK,
    MOD_DQPSK,
    MOD_8PSK,
    MOD_D8PSK,
    MOD_16QAM,
    MOD_UNKNOWN = 255,
}

#[derive(Debug, Copy, Clone)]
pub struct TExtendedAddress (pub [u8; 8]);

// #[repr(C)]
// #[derive(Copy, Clone)]
// pub union TAddress {
//     short_addr: u16,
//     extended_addr: TAdpExtendedAddress,
// }

// #[derive(Copy, Clone)]
// pub struct TAdpAddress {
//     mu8AddrSize: u8,
//     address: TAddress,
// }
// impl TAddress {
//     pub fn new()
// }

#[derive(Debug, Copy, Clone)]
pub enum TAddress {
    Short(u16),
    Extended(TExtendedAddress)
}
impl Into<Vec<u8>> for TAddress {
    fn into(self) -> Vec<u8> {
        let mut v = Vec::new();

        match self {
            Self::Short(a) => {
                v.append(&mut a.to_be_bytes().to_vec());
            },
            Self::Extended(e) => {
                v.append(&mut e.0.to_vec());
            }
        }
        return v
    }
}
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TAdpBand {
    ADP_BAND_CENELEC_A = 0,
    ADP_BAND_CENELEC_B = 1,
    ADP_BAND_FCC = 2,
    ADP_BAND_ARIB = 3,
}

/**********************************************************************************************************************/
/** PAN descriptor structure specification
 *
 ***********************************************************************************************************************
 * @param u16PanId The 16-bit PAN identifier.
 * @param u8LinkQuality The 8-bit link quality of LBA.
 * @param u16LbaAddress The 16 bit short address of a device in this PAN to be used as the LBA by the associating device.
 * @param u16RcCoord The estimated route cost from LBA to the coordinator.
 **********************************************************************************************************************/
#[derive(Debug)]
pub struct TAdpPanDescriptor {
    pub pan_id: u16,
    pub link_quality: u8,
    pub lba_address: u16,
    pub rc_coord: u16,
}
/**********************************************************************************************************************/
/** Path discovery
 *
 ***********************************************************************************************************************
 * @param m_u16HopAddress The hop / node address
 * @param m_u8Mns MetricNotSupported: 1 the metric type is not supported by the hop, 0 if supported
 * @param  m_u8LinkCost LinkCost of the node
 **********************************************************************************************************************/
struct THopDescriptor {
    m_u16HopAddress: u16,
    m_u8Mns: u8,
    m_u8LinkCost: u8,
}

/**********************************************************************************************************************/
/** Path discovery
 *
 ***********************************************************************************************************************
 * @param m_u16DstAddr The short unicast destination address of the path discovery.
 * @param m_u16ExpectedOrigAddr The expected originator of the path reply
 * @param m_u16OrigAddr The real originator of the path reply
 * @param m_u8MetricType Path metric type
 * @param m_u8ForwardHopsCount Number of path hops in the forward table
 * @param m_u8ReverseHopsCount Number of path hops in the reverse table
 * @param m_aForwardPath Table with the information of each hop in forward direction (according to m_u8ForwardHopsCount)
 * @param m_aReversePath Table with the information of each hop in reverse direction (according to m_u8ReverseHopsCount)
 **********************************************************************************************************************/
pub struct TPathDescriptor {
    m_u16DstAddr: u16,
    m_u16ExpectedOrigAddr: u16,
    m_u16OrigAddr: u16,
    m_u8MetricType: u8,
    m_u8ForwardHopsCount: u8,
    m_u8ReverseHopsCount: u8,
    m_aForwardPath: [THopDescriptor; 16],
    m_aReversePath: [THopDescriptor; 16],
}
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive, Hash)]
#[repr(u32)]
pub enum EAdpPibAttribute {
    ADP_IB_SECURITY_LEVEL = 0x00000000,
    ADP_IB_PREFIX_TABLE = 0x00000001,
    ADP_IB_BROADCAST_LOG_TABLE_ENTRY_TTL = 0x00000002,
    ADP_IB_METRIC_TYPE = 0x00000003,
    ADP_IB_LOW_LQI_VALUE = 0x00000004,
    ADP_IB_HIGH_LQI_VALUE = 0x00000005,
    ADP_IB_RREP_WAIT = 0x00000006,
    ADP_IB_CONTEXT_INFORMATION_TABLE = 0x00000007,
    ADP_IB_COORD_SHORT_ADDRESS = 0x00000008,
    ADP_IB_RLC_ENABLED = 0x00000009,
    ADP_IB_ADD_REV_LINK_COST = 0x0000000A,
    ADP_IB_BROADCAST_LOG_TABLE = 0x0000000B,
    ADP_IB_ROUTING_TABLE = 0x0000000C,
    ADP_IB_UNICAST_RREQ_GEN_ENABLE = 0x0000000D,
    ADP_IB_GROUP_TABLE = 0x0000000E,
    ADP_IB_MAX_HOPS = 0x0000000F,
    ADP_IB_DEVICE_TYPE = 0x00000010,
    ADP_IB_NET_TRAVERSAL_TIME = 0x00000011,
    ADP_IB_ROUTING_TABLE_ENTRY_TTL = 0x00000012,
    ADP_IB_KR = 0x00000013,
    ADP_IB_KM = 0x00000014,
    ADP_IB_KC = 0x00000015,
    ADP_IB_KQ = 0x00000016,
    ADP_IB_KH = 0x00000017,
    ADP_IB_RREQ_RETRIES = 0x00000018,
    // ADP_IB_RREQ_RERR_WAIT = 0x00000019, SPEC 15
    ADP_IB_RREQ_WAIT = 0x00000019,
    ADP_IB_WEAK_LQI_VALUE = 0x0000001A,
    ADP_IB_KRT = 0x0000001B,
    ADP_IB_SOFT_VERSION = 0x0000001C,
    ADP_IB_SNIFFER_MODE = 0x0000001D,
    ADP_IB_BLACKLIST_TABLE = 0x0000001E,
    ADP_IB_BLACKLIST_TABLE_ENTRY_TTL = 0x0000001F,
    ADP_IB_MAX_JOIN_WAIT_TIME = 0x00000020,
    ADP_IB_PATH_DISCOVERY_TIME = 0x00000021,
    ADP_IB_ACTIVE_KEY_INDEX = 0x00000022,
    ADP_IB_DESTINATION_ADDRESS_SET = 0x00000023,
    ADP_IB_DEFAULT_COORD_ROUTE_ENABLED = 0x00000024,
    ADP_IB_DISABLE_DEFAULT_ROUTING = 0x000000F0,
    // manufacturer
    ADP_IB_MANUF_REASSEMBY_TIMER = 0x080000C0,
    ADP_IB_MANUF_IPV6_HEADER_COMPRESSION = 0x080000C1,
    ADP_IB_MANUF_EAP_PRESHARED_KEY = 0x080000C2,
    ADP_IB_MANUF_EAP_NETWORK_ACCESS_IDENTIFIER = 0x080000C3,
    ADP_IB_MANUF_BROADCAST_SEQUENCE_NUMBER = 0x080000C4,
    ADP_IB_MANUF_REGISTER_DEVICE = 0x080000C5,
    ADP_IB_MANUF_DATAGRAM_TAG = 0x080000C6,
    ADP_IB_MANUF_RANDP = 0x080000C7,
    ADP_IB_MANUF_ROUTING_TABLE_COUNT = 0x080000C8,
    ADP_IB_MANUF_DISCOVER_SEQUENCE_NUMBER = 0x080000C9,
    ADP_IB_MANUF_FORCED_NO_ACK_REQUEST = 0x080000CA,
    ADP_IB_MANUF_LQI_TO_COORD = 0x080000CB,
    ADP_IB_MANUF_BROADCAST_ROUTE_ALL = 0x080000CC,
    ADP_IB_MANUF_KEEP_PARAMS_AFTER_KICK_LEAVE = 0x080000CD,
    ADP_IB_MANUF_ADP_INTERNAL_VERSION = 0x080000CE,
    ADP_IB_MANUF_CIRCULAR_ROUTES_DETECTED = 0x080000CF,
    ADP_IB_MANUF_LAST_CIRCULAR_ROUTE_ADDRESS = 0x080000D0,
    ADP_IB_MANUF_IPV6_ULA_DEST_SHORT_ADDRESS = 0x080000D1,
    ADP_IB_MANUF_MAX_REPAIR_RESEND_ATTEMPTS = 0x080000D2,
    ADP_IB_MANUF_DISABLE_AUTO_RREQ = 0x080000D3,
    ADP_IB_MANUF_ALL_NEIGHBORS_BLACKLISTED_COUNT = 0x080000D5,
    ADP_IB_MANUF_QUEUED_ENTRIES_REMOVED_TIMEOUT_COUNT = 0x080000D6,
    ADP_IB_MANUF_QUEUED_ENTRIES_REMOVED_ROUTE_ERROR_COUNT = 0x080000D7,
    ADP_IB_MANUF_PENDING_DATA_IND_SHORT_ADDRESS = 0x080000D8,
    ADP_IB_MANUF_GET_BAND_CONTEXT_TONES = 0x080000D9,
    ADP_IB_MANUF_UPDATE_NON_VOLATILE_DATA = 0x080000DA,
    ADP_IB_MANUF_DISCOVER_ROUTE_GLOBAL_SEQ_NUM = 0x080000DB,

    INVALID = 0xFFFFFFFF
}
#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive, Hash)]
#[repr(u32)]
pub enum EMacWrpPibAttribute {
    MAC_WRP_PIB_ACK_WAIT_DURATION = 0x00000040,
    MAC_WRP_PIB_MAX_BE = 0x00000047,
    MAC_WRP_PIB_BSN = 0x00000049,
    MAC_WRP_PIB_DSN = 0x0000004C,
    MAC_WRP_PIB_MAX_CSMA_BACKOFFS = 0x0000004E,
    MAC_WRP_PIB_MIN_BE = 0x0000004F,
    MAC_WRP_PIB_PAN_ID = 0x00000050,
    MAC_WRP_PIB_PROMISCUOUS_MODE = 0x00000051,
    MAC_WRP_PIB_SHORT_ADDRESS = 0x00000053,
    MAC_WRP_PIB_MAX_FRAME_RETRIES = 0x00000059,
    MAC_WRP_PIB_TIMESTAMP_SUPPORTED = 0x0000005C,
    MAC_WRP_PIB_SECURITY_ENABLED = 0x0000005D,
    MAC_WRP_PIB_KEY_TABLE = 0x00000071,
    MAC_WRP_PIB_FRAME_COUNTER = 0x00000077,
    MAC_WRP_PIB_HIGH_PRIORITY_WINDOW_SIZE = 0x00000100,
    MAC_WRP_PIB_TX_DATA_PACKET_COUNT = 0x00000101,
    MAC_WRP_PIB_RX_DATA_PACKET_COUNT = 0x00000102,
    MAC_WRP_PIB_TX_CMD_PACKET_COUNT = 0x00000103,
    MAC_WRP_PIB_RX_CMD_PACKET_COUNT = 0x00000104,
    MAC_WRP_PIB_CSMA_FAIL_COUNT = 0x00000105,
    MAC_WRP_PIB_CSMA_NO_ACK_COUNT = 0x00000106,
    MAC_WRP_PIB_RX_DATA_BROADCAST_COUNT = 0x00000107,
    MAC_WRP_PIB_TX_DATA_BROADCAST_COUNT = 0x00000108,
    MAC_WRP_PIB_BAD_CRC_COUNT = 0x00000109,
    MAC_WRP_PIB_NEIGHBOUR_TABLE = 0x0000010A,
    MAC_WRP_PIB_FREQ_NOTCHING = 0x0000010B,
    MAC_WRP_PIB_CSMA_FAIRNESS_LIMIT = 0x0000010C,
    MAC_WRP_PIB_TMR_TTL = 0x0000010D,
    MAC_WRP_PIB_NEIGHBOUR_TABLE_ENTRY_TTL = 0x0000010E, // Used in Spec15
    // MAC_WRP_PIB_POS_TABLE_ENTRY_TTL = 0x0000010E,       // Used in Spec17
    MAC_WRP_PIB_RC_COORD = 0x0000010F,
    MAC_WRP_PIB_TONE_MASK = 0x00000110,
    MAC_WRP_PIB_BEACON_RANDOMIZATION_WINDOW_LENGTH = 0x00000111,
    MAC_WRP_PIB_A = 0x00000112,
    MAC_WRP_PIB_K = 0x00000113,
    MAC_WRP_PIB_MIN_CW_ATTEMPTS = 0x00000114,
    MAC_WRP_PIB_CENELEC_LEGACY_MODE = 0x00000115,
    MAC_WRP_PIB_FCC_LEGACY_MODE = 0x00000116,
    MAC_WRP_PIB_BROADCAST_MAX_CW_ENABLE = 0x0000011E,
    MAC_WRP_PIB_TRANSMIT_ATTEN = 0x0000011F,
    MAC_WRP_PIB_POS_TABLE = 0x00000120,
    // manufacturer specific
    // provides access to device table
    MAC_WRP_PIB_MANUF_DEVICE_TABLE = 0x08000000,
    // Extended address of this node.
    MAC_WRP_PIB_MANUF_EXTENDED_ADDRESS = 0x08000001,
    // provides access to neighbour table by short address (transmitted as index)
    MAC_WRP_PIB_MANUF_NEIGHBOUR_TABLE_ELEMENT = 0x08000002,
    // returns the maximum number of tones used by the band
    MAC_WRP_PIB_MANUF_BAND_INFORMATION = 0x08000003,
    // Short address of the coordinator.
    MAC_WRP_PIB_MANUF_COORD_SHORT_ADDRESS = 0x08000004,
    // Maximal payload supported by MAC.
    MAC_WRP_PIB_MANUF_MAX_MAC_PAYLOAD_SIZE = 0x08000005,
    // Resets the device table upon a GMK activation.
    MAC_WRP_PIB_MANUF_SECURITY_RESET = 0x08000006,
    // Forces Modulation Scheme in every transmitted frame
    // 0 - Not forced, 1 - Force Differential, 2 - Force Coherent
    MAC_WRP_PIB_MANUF_FORCED_MOD_SCHEME = 0x08000007,
    // Forces Modulation Type in every transmitted frame
    // 0 - Not forced, 1 - Force BPSK_ROBO, 2 - Force BPSK, 3 - Force QPSK, 4 - Force 8PSK
    MAC_WRP_PIB_MANUF_FORCED_MOD_TYPE = 0x08000008,
    // Forces ToneMap in every transmitted frame
    // {0} - Not forced, other value will be used as tonemap
    MAC_WRP_PIB_MANUF_FORCED_TONEMAP = 0x08000009,
    // Forces Modulation Scheme bit in Tone Map Response
    // 0 - Not forced, 1 - Force Differential, 2 - Force Coherent
    MAC_WRP_PIB_MANUF_FORCED_MOD_SCHEME_ON_TMRESPONSE = 0x0800000A,
    // Forces Modulation Type bits in Tone Map Response
    // 0 - Not forced, 1 - Force BPSK_ROBO, 2 - Force BPSK, 3 - Force QPSK, 4 - Force 8PSK
    MAC_WRP_PIB_MANUF_FORCED_MOD_TYPE_ON_TMRESPONSE = 0x0800000B,
    // Forces ToneMap field Tone Map Response
    // {0} - Not forced, other value will be used as tonemap field
    MAC_WRP_PIB_MANUF_FORCED_TONEMAP_ON_TMRESPONSE = 0x0800000C,
    // Gets Modulation Scheme of last received frame
    MAC_WRP_PIB_MANUF_LAST_RX_MOD_SCHEME = 0x0800000D,
    // Gets Modulation Scheme of last received frame
    MAC_WRP_PIB_MANUF_LAST_RX_MOD_TYPE = 0x0800000E,
    // Indicates whether an LBP frame for other destination has been received
    MAC_WRP_PIB_MANUF_LBP_FRAME_RECEIVED = 0x0800000F,
    // Indicates whether an LBP frame for other destination has been received
    MAC_WRP_PIB_MANUF_LNG_FRAME_RECEIVED = 0x08000010,
    // Indicates whether an Beacon frame from other nodes has been received
    MAC_WRP_PIB_MANUF_BCN_FRAME_RECEIVED = 0x08000011,
    // Gets number of valid elements in the Neighbour Table
    MAC_WRP_PIB_MANUF_NEIGHBOUR_TABLE_COUNT = 0x08000012,
    // Gets number of discarded packets due to Other Destination
    MAC_WRP_PIB_MANUF_RX_OTHER_DESTINATION_COUNT = 0x08000013,
    // Gets number of discarded packets due to Invalid Frame Lenght
    MAC_WRP_PIB_MANUF_RX_INVALID_FRAME_LENGTH_COUNT = 0x08000014,
    // Gets number of discarded packets due to MAC Repetition
    MAC_WRP_PIB_MANUF_RX_MAC_REPETITION_COUNT = 0x08000015,
    // Gets number of discarded packets due to Wrong Addressing Mode
    MAC_WRP_PIB_MANUF_RX_WRONG_ADDR_MODE_COUNT = 0x08000016,
    // Gets number of discarded packets due to Unsupported Security
    MAC_WRP_PIB_MANUF_RX_UNSUPPORTED_SECURITY_COUNT = 0x08000017,
    // Gets number of discarded packets due to Wrong Key Id
    MAC_WRP_PIB_MANUF_RX_WRONG_KEY_ID_COUNT = 0x08000018,
    // Gets number of discarded packets due to Invalid Key
    MAC_WRP_PIB_MANUF_RX_INVALID_KEY_COUNT = 0x08000019,
    // Gets number of discarded packets due to Wrong Frame Counter
    MAC_WRP_PIB_MANUF_RX_WRONG_FC_COUNT = 0x0800001A,
    // Gets number of discarded packets due to Decryption Error
    MAC_WRP_PIB_MANUF_RX_DECRYPTION_ERROR_COUNT = 0x0800001B,
    // Gets number of discarded packets due to Segment Decode Error
    MAC_WRP_PIB_MANUF_RX_SEGMENT_DECODE_ERROR_COUNT = 0x0800001C,
    // Enables MAC Sniffer
    MAC_WRP_PIB_MANUF_ENABLE_MAC_SNIFFER = 0x0800001D,
    // Gets number of valid elements in the POS Table. Unused in SPEC-15
    MAC_WRP_PIB_MANUF_POS_TABLE_COUNT = 0x0800001E,
    // Gets or Sets number of retires left before forcing ROBO mode
    MAC_WRP_PIB_MANUF_RETRIES_LEFT_TO_FORCE_ROBO = 0x0800001F,
    // Gets internal MAC version
    MAC_WRP_PIB_MANUF_MAC_INTERNAL_VERSION = 0x08000021,
    // Gets internal MAC RT version
    MAC_WRP_PIB_MANUF_MAC_RT_INTERNAL_VERSION = 0x08000022,
    // Gets or sets a parameter in Phy layer. Index will be used to contain PHY parameter ID.
    // See definitions below
    MAC_WRP_PIB_MANUF_PHY_PARAM = 0x08000020,
}

pub fn usi_message_to_message(msg: &InMessage) -> Option<Message> {
    {
        if let Some(cmd) = msg.buf.get(0) {
            match *cmd {
                G3_SERIAL_MSG_ADP_DATA_CONFIRM => {
                    if let Some(data_response) = DataResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::DataResponse(data_response)));
                    }
                },
                G3_SERIAL_MSG_ADP_SET_CONFIRM => {
                    if let Some(set_response) = SetResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::SetResponse(set_response)));
                    }
                },
                G3_SERIAL_MSG_ADP_MAC_SET_CONFIRM => {
                    if let Some(set_response) = SetMacResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::SetMacResponse(set_response)));
                    }
                },
                G3_SERIAL_MSG_STATUS => {
                    if let Some(msg_response) = MsgStatusResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::MsgStatusResponse(msg_response)));
                    }
                },
                G3_SERIAL_MSG_ADP_DISCOVERY_INDICATION => {
                    if let Some(discovery_event) = DiscoveryEvent::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::DiscoveryEvent(discovery_event)));
                    }
                },
                G3_SERIAL_MSG_ADP_DISCOVERY_CONFIRM => {
                    if let Some(discovery_response) = DiscoveryResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::DiscoveryResponse(discovery_response)));
                    }
                },
                G3_SERIAL_MSG_ADP_GET_CONFIRM => {
                    if let Some(get_response) = GetResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::GetResponse(get_response)));
                    }
                },
                G3_SERIAL_MSG_ADP_MAC_GET_CONFIRM => {
                    if let Some(mac_get_response) = GetMacResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::GetMacResponse(mac_get_response)));
                    }

                },
                G3_SERIAL_MSG_ADP_NETWORK_START_CONFIRM=> {
                    if let Some(network_start_response) = NetworkStartResponse::try_from_message(&msg){
                        return Some(Message::AdpG3(AdpG3::NetworkStartResponse(network_start_response)));
                    }
                }
                _ => return None,
            }
        }
        None
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum EAdpStatus {
    /// Success
    G3_SUCCESS = 0x00,
    /// Invalid request
    G3_INVALID_REQUEST = 0xA1,
    /// Request failed
    G3_FAILED = 0xA2,
    /// Invalid IPv6 frame
    G3_INVALID_IPV6_FRAME = 0xA3,
    /// Not permited
    G3_NOT_PERMITED = 0xA4,
    /// No route to destination
    G3_ROUTE_ERROR = 0xA5,
    /// Operation timed out
    G3_TIMEOUT = 0xA6,
    /// An attempt to write to a MAC PIB attribute that is in a table failed because the specified table index was out of range.
    G3_INVALID_INDEX = 0xA7,
    /// A parameter in the primitive is either not supported or is out of the valid range.
    G3_INVALID_PARAMETER = 0xA8,
    /// A scan operation failed to find any network beacons.
    G3_NO_BEACON = 0xA9,
    /// A SET/GET request was issued with the identifier of an attribute that is read only.
    G3_READ_ONLY = 0xB0,
    /// A SET/GET request was issued with the identifier of a PIB attribute that is not supported.
    G3_UNSUPPORTED_ATTRIBUTE = 0xB1,
    /// The path discovery has only a part of the path to its desired final destination.
    G3_INCOMPLETE_PATH = 0xB2,
    /// Busy: operation already in progress.
    G3_BUSY = 0xB3,
    /// Not enough resources
    G3_NO_BUFFERS = 0xB4,
    /// Error internal
    G3_ERROR_INTERNAL = 0xFF,
}

struct TAdpRouteNotFoundIndication {
    m_u16SrcAddr: u16,
    m_u16DestAddr: u16,
    m_u16NextHopAddr: u16,
    m_u16PreviousHopAddr: u16,
    m_u16RouteCost: u16,
    m_u8HopCount: u8,
    m_u8WeakLinkCount: bool,
    m_bRouteJustBroken: bool,
    m_bCompressedHeader: bool,
    m_u16NsduLength: u16,
    m_pNsdu: Vec<u8>,
}

#[derive(Debug)]
pub enum Message {
    //    SnifPrime,
    //    MacPrime,
    //    MlmePrime,
    //     PlmePrime,
    //     _432_PRIME,
    //     BasemngPrime,
    //     PRIMEoUDP,
    //     PhyAtpl2x0,
    //     ATPL230,
    //     ATPL250,
    // SnifG3,
    // MacG3,
    AdpG3(AdpG3),
    // CoordG3,
    // PrimeApi,
    // UserDefined,
    // UserDefined2,
    // INVALID
}

#[derive(Debug)]
pub enum AdpG3 {
    MsgStatusResponse(MsgStatusResponse),
    DataResponse(DataResponse),
    DataEvent(DataEvent),
    NetworkStatusEvent(NetworkStatusEvent),
    DiscoveryResponse(DiscoveryResponse),
    NetworkStartResponse(NetworkStartResponse),
    NetworkJoinResponse(NetworkJoinResponse),
    NetworkLeaveResponse(NetworkLeaveResponse),
    NetworkLeaveEvent(NetworkLeaveEvent),
    ResetResponse(ResetResponse),
    SetResponse(SetResponse),
    GetResponse(GetResponse),
    LbpReponse(LbpReponse),
    LbpEvent(LbpEvent),
    RouteDiscoveryResponse(RouteDiscoveryResponse),
    PathDiscoveryResponse(PathDiscoveryResponse),
    SetMacResponse(SetMacResponse),
    GetMacResponse(GetMacResponse),
    BufferEvent(BufferEvent),
    DiscoveryEvent(DiscoveryEvent),
    PreqEvent(PreqEvent),
    UpdNonVolatileDataEvent(UpdNonVolatileDataEvent),
    RouteNotFoundEvent(RouteNotFoundEvent),
}
#[derive(Debug)]
pub struct MsgStatusResponse {
    status: EAdpStatus,
    cmd: u8,
}

impl MsgStatusResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<MsgStatusResponse> {
        if msg.buf.len() > 0 {
            //cmd is the first byte???
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[0]) {
                if let Some(&cmd) = msg.buf.get(1) {
                    return Some(MsgStatusResponse { status, cmd });
                }
            }
        }
        None
    }
}

pub struct GetResponse {
    pub status: EAdpStatus,
    pub attribute_id: u32,
    pub attribute_idx: u16,
    pub attribute_len: u8,
    pub attribute_val: Vec<u8>,
}

const MIN_GET_RESPONSE_LEN: usize = 8;
impl GetResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<GetResponse> {
        if msg.buf.len() >= MIN_GET_RESPONSE_LEN + 1 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[1]) {
                
                let mut attribute_id:u32 = msg.buf[2] as u32;
                attribute_id = (attribute_id << 8) + msg.buf[3] as u32;
                attribute_id = (attribute_id << 8) + msg.buf[4] as u32;
                attribute_id = (attribute_id << 8) + msg.buf[5] as u32;
                
                let mut attribute_idx = msg.buf[6] as u16;
                attribute_idx = (attribute_idx << 8) + msg.buf[7] as u16;
                // let mut attribute_id;
                // if let Some(attribute_id_buf) = msg.buf.get(2..5) {
                //     attribute_id = u32::from_be_bytes(*attribute_id_buf);
                // }
                
                // let mut attribute_idx;
                // if let Some(attribute_idx_buf) = msg.buf.get(5..7) {
                //     attribute_idx = u16::from_be_bytes(attribute_idx_buf);
                // }
                let attribute_len = msg.buf[8];
                let mut result = GetResponse {
                    status, attribute_id, attribute_idx, attribute_len, attribute_val: Vec::new()
                };

                if (attribute_len > 0 && msg.buf.len() >= (MIN_GET_RESPONSE_LEN + 1 + attribute_len as usize)) {
                    if let Some(content) = msg.buf.get(9..){
                        result.attribute_val.append(&mut content.to_vec());
                    }                                        
                }
                return Some(result);
            }
        }
        None
    }
}

impl fmt::Debug for GetResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetResponse")
            .field("status", &self.status)
            .field("attribute id", &EAdpPibAttribute::try_from_primitive(self.attribute_id).unwrap_or(EAdpPibAttribute::INVALID))
            .field("attribute index", &self.attribute_idx)
            .field("attribute value", &self.attribute_val)
            .finish()
    }
}

#[derive(Debug)]
pub struct GetMacResponse {
    pub status: EAdpStatus,
    pub attribute_id: u32,
    pub attribute_idx: u16,
    pub attribute_len: u8,
    pub attribute_val: Vec<u8>,
}

const MIN_GET_MAC_RESPONSE_LEN: usize = 8;
impl GetMacResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<GetMacResponse> {
        if msg.buf.len() >= MIN_GET_RESPONSE_LEN + 1 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[1]) {
                
                let mut attribute_id:u32 = msg.buf[2] as u32;
                attribute_id = (attribute_id << 8) + msg.buf[3] as u32;
                attribute_id = (attribute_id << 8) + msg.buf[4] as u32;
                attribute_id = (attribute_id << 8) + msg.buf[5] as u32;
                
                let mut attribute_idx = msg.buf[6] as u16;
                attribute_idx = (attribute_idx << 8) + msg.buf[7] as u16;
                // let mut attribute_id;
                // if let Some(attribute_id_buf) = msg.buf.get(2..5) {
                //     attribute_id = u32::from_be_bytes(*attribute_id_buf);
                // }
                
                // let mut attribute_idx;
                // if let Some(attribute_idx_buf) = msg.buf.get(5..7) {
                //     attribute_idx = u16::from_be_bytes(attribute_idx_buf);
                // }
                let attribute_len = msg.buf[8];
                let mut result = GetMacResponse {
                    status, attribute_id, attribute_idx, attribute_len, attribute_val: Vec::new()
                };

                if (attribute_len > 0 && msg.buf.len() >= (MIN_GET_RESPONSE_LEN + 1 + attribute_len as usize)) {
                    if let Some(content) = msg.buf.get(9..){
                        result.attribute_val.append(&mut content.to_vec());
                    }                                        
                }
                return Some(result);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct LbpReponse {}

#[derive(Debug)]
pub struct LbpEvent {}

#[derive(Debug)]
pub struct RouteDiscoveryResponse {}

#[derive(Debug)]
pub struct PathDiscoveryResponse {}

#[derive(Debug)]
pub struct BufferEvent {}

const DISCOVERY_EVENT_LEN: usize = 7;

#[derive(Debug)]
pub struct DiscoveryEvent {
    pub pan_descriptor: TAdpPanDescriptor,
}

impl DiscoveryEvent {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<DiscoveryEvent> {
        if msg.buf.len() == DISCOVERY_EVENT_LEN + 1 {
            //Add one byte for cmd
            let pan_id = (msg.buf[1] as u16) << 8 | (msg.buf[2] as u16);
            let link_quality = msg.buf[3];
            let lba_address = (msg.buf[4] as u16) << 8 | (msg.buf[5] as u16);
            let rc_coord = (msg.buf[6] as u16) << 8 | (msg.buf[7] as u16);
            return Some(DiscoveryEvent {
                pan_descriptor: TAdpPanDescriptor {
                    pan_id,
                    link_quality,
                    lba_address,
                    rc_coord,
                },
            });
        }
        None
    }
}

#[derive(Debug)]
pub struct PreqEvent {}

#[derive(Debug)]
pub struct UpdNonVolatileDataEvent {}

#[derive(Debug)]
pub struct RouteNotFoundEvent {}

#[derive(Debug)]
pub struct NetworkJoinResponse {}

#[derive(Debug)]
pub struct NetworkLeaveEvent {}

#[derive(Debug)]
pub struct NetworkLeaveResponse {}

#[derive(Debug)]
pub struct ResetResponse {}

#[derive(Debug)]
pub struct SetMacResponse {
    status: EAdpStatus,
    attribute_id: u32,
    attribute_idx: u16,
}
impl SetMacResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<SetMacResponse> {
        if msg.buf.len() == SET_RESPONSE_LEN + 1 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[1]) {
                return Some(SetMacResponse {
                    status,
                    attribute_id: (msg.buf[2] as u32) << 24
                        | (msg.buf[3] as u32) << 16
                        | (msg.buf[4] as u32) << 8
                        | (msg.buf[5] as u32),
                    attribute_idx: (msg.buf[6] as u16) << 8 | (msg.buf[7] as u16),
                });
            }
        }
        None
    }
}

const SET_RESPONSE_LEN: usize = 7;
pub struct SetResponse {
    status: EAdpStatus,
    attribute_id: u32,
    attribute_idx: u16,
}

impl SetResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<SetResponse> {
        if msg.buf.len() == SET_RESPONSE_LEN + 1 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[1]) {
                return Some(SetResponse {
                    status,
                    attribute_id: (msg.buf[2] as u32) << 24
                        | (msg.buf[3] as u32) << 16
                        | (msg.buf[4] as u32) << 8
                        | (msg.buf[5] as u32),
                    attribute_idx: (msg.buf[6] as u16) << 8 | (msg.buf[7] as u16),
                });
            }
        }
        None
    }
}
impl fmt::Debug for SetResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SetResponse")
            .field("status", &self.status)
            .field("attribute id", &EAdpPibAttribute::try_from_primitive(self.attribute_id).unwrap_or(EAdpPibAttribute::INVALID))
            .field("attribute index", &self.attribute_idx)
            .finish()
    }
}

#[derive(Debug)]
pub struct NetworkStartResponse {
    status: EAdpStatus,
}
impl NetworkStartResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<NetworkStartResponse> {
        if msg.buf.len() > 1 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[1]) {
                return Some(NetworkStartResponse { status });
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct DiscoveryResponse {
    status: EAdpStatus,
}
impl DiscoveryResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<DiscoveryResponse> {
        if msg.buf.len() > 1 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[1]) {
                return Some(DiscoveryResponse { status });
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct NetworkStatusEvent {}
pub struct DataResponse {
    status: EAdpStatus,
    nsdu_handle: u8,
}
impl DataResponse {
    pub fn try_from_message(msg: &usi::InMessage) -> Option<DataResponse> {
        if let (Some(&status8), Some(&nsdu_handle)) = (msg.buf.get(1), msg.buf.get(2)) {
            if let Ok(status) = EAdpStatus::try_from(status8) {
                return Some(DataResponse {
                    status,
                    nsdu_handle,
                });
            }
        }
        None
    }
}
impl fmt::Debug for DataResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataResponse")
            .field("status", &self.status)
            .field("nsdu handle", &self.nsdu_handle)
            .finish()
    }
}

#[derive(Debug)]
pub struct DataEvent {
    nsdu: Vec<u8>,
    link_quality_indicator: u8,
}

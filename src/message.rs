use usi::UsiMessage;
use crate::usi;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

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

use std::fmt;

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
pub struct TAdpExtendedAddress {
    m_au8Value: [u8; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union TAddress {
    m_u16ShortAddr: u16,
    m_ExtendedAddress: TAdpExtendedAddress,
}

#[derive(Copy, Clone)]
pub struct TAdpAddress {
    mu8AddrSize: u8,
    address: TAddress,
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
}


pub fn usi_message_to_message (msg: &UsiMessage) -> Option<Message> {
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
				G3_SERIAL_MSG_STATUS => {
					if let Some(msg_response) = MsgStatusResponse::try_from_message(&msg){
                        return Some(Message::AdpG3(AdpG3::MsgStatusResponse(msg_response)));
					}
				},
				G3_SERIAL_MSG_ADP_DISCOVERY_INDICATION => {
					if let Some(discovery_event) = DiscoveryEvent::try_from_message(&msg){
                        return Some(Message::AdpG3(AdpG3::DiscoveryEvent(discovery_event)));
					}
				},
				G3_SERIAL_MSG_ADP_DISCOVERY_CONFIRM => {
					if let Some(discovery_response) = DiscoveryResponse::try_from_message(&msg) {
                        return Some(Message::AdpG3(AdpG3::DiscoveryResponse(discovery_response)));
					}
				},
				_ => {
                    return None
				}
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
impl fmt::Debug for TAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TAddress")
            .field("short", unsafe { &self.m_u16ShortAddr })
            .field("extended", unsafe { &self.m_ExtendedAddress })
            .finish()
    }
}

impl fmt::Debug for TAdpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TAdpAddress")
            .field("size", &self.mu8AddrSize)
            .field("address", &self.address)
            .finish()
    }
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
    MacSetResponse(MacSetResponse),
    MacGetResponse(MacGetResponse),
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
    pub fn try_from_message(msg: &usi::UsiMessage) -> Option<MsgStatusResponse> {
        if msg.buf.len() > 0 {
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

#[derive(Debug)]
pub struct GetResponse {
    status: EAdpStatus,
    attribute_id: u32,
    attribute_idx: u16,
    attribute_len: u8,
    attribute_val: [u8; 64],
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
pub struct MacSetResponse {}

#[derive(Debug)]
pub struct MacGetResponse {}

#[derive(Debug)]
pub struct BufferEvent {}

const DISCOVERY_EVENT_LEN: usize = 7;

#[derive(Debug)]
pub struct DiscoveryEvent {
    pub pan_descriptor: TAdpPanDescriptor,
}

impl DiscoveryEvent {
    pub fn try_from_message(msg: &usi::UsiMessage) -> Option<DiscoveryEvent> {
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

const SET_RESPONSE_LEN: usize = 7;
pub struct SetResponse {
    status: EAdpStatus,
    attribute_id: u32,
    attribute_idx: u16,
}

impl SetResponse {
    pub fn try_from_message(msg: &usi::UsiMessage) -> Option<SetResponse> {
        if msg.buf.len() == SET_RESPONSE_LEN + 1 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[0]) {
                return Some(SetResponse {
                    status,
                    attribute_id: (msg.buf[1] as u32) << 24
                        | (msg.buf[2] as u32) << 16
                        | (msg.buf[3] as u32) << 8
                        | (msg.buf[4] as u32),
                    attribute_idx: (msg.buf[5] as u16) << 8 | (msg.buf[6] as u16),
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
            .field("attribute id", &self.attribute_id)
            .field("attribute index", &self.attribute_idx)
            .finish()
    }
}

#[derive(Debug)]
pub struct NetworkStartResponse {}

#[derive(Debug)]
pub struct DiscoveryResponse {
    status: EAdpStatus,
}
impl DiscoveryResponse {
    pub fn try_from_message(msg: &usi::UsiMessage) -> Option<DiscoveryResponse> {
        if msg.buf.len() > 0 {
            //Add one byte for cmd
            if let Ok(status) = EAdpStatus::try_from(msg.buf[0]) {
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
    pub fn try_from_message(msg: &usi::UsiMessage) -> Option<DataResponse> {
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

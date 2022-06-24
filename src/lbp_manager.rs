use core::ops::Deref;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;

use std::net::IpAddr;
use std::rc::Rc;
use std::time::Instant;

use crate::adp::AdpG3LbpReponse;
use crate::adp::EAdpStatus;
use crate::adp::TAdpBand;
use crate::adp::TExtendedAddress;
use crate::app_config;
use crate::lbp;
use crate::lbp::JoiningMessage;
use crate::lbp_functions::*;
use crate::request;

const UC_MESSAGE_TIMEOUT_MS: u128 = 40_000;

const CONF_PARAM_SHORT_ADDR: u8 = 0x1D;
const CONF_PARAM_GMK: u8 = 0x27;
const CONF_PARAM_GMK_ACTIVATION: u8 = 0x2B;
const CONF_PARAM_GMK_REMOVAL: u8 = 0x2F;
const CONF_PARAM_RESULT: u8 = 0x31;

#[derive(PartialEq, Eq, Debug)]
enum DeviceState {
    BS_STATE_WAITING_JOINNING = 0,
    BS_STATE_SENT_EAP_MSG_1,
    BS_STATE_WAITING_EAP_MSG_2,
    BS_STATE_SENT_EAP_MSG_3,
    BS_STATE_WAITING_EAP_MSG_4,
    BS_STATE_SENT_EAP_MSG_ACCEPTED,
    BS_STATE_SENT_EAP_MSG_DECLINED,
}

#[derive(Debug)]
pub struct DeviceSlot {
    state: DeviceState,
    m_lbd_address: TExtendedAddress,
    us_lba_src_addr: u16,
    us_assigned_short_address: u16,
    uc_tx_handle: u8,
    ul_timeout: u128,
    uc_tx_attemps: u8,
    m_rand_s: TEapPskRand,
    uc_pending_confirms: u8,
    uc_pending_tx_handler: u8,
    m_psk_context: TEapPskContext,
    data: Option<Vec<u8>>,
}
impl DeviceSlot {
    pub fn new(ext_addr: TExtendedAddress, short_address: u16) -> Self {
        DeviceSlot {
            state: DeviceState::BS_STATE_WAITING_JOINNING,
            m_lbd_address: ext_addr,
            us_lba_src_addr: 0,
            us_assigned_short_address: short_address,
            uc_tx_handle: 0xff,
            ul_timeout: 0,
            uc_tx_attemps: 0,
            m_rand_s: TEapPskRand::new(),
            uc_pending_confirms: 0,
            uc_pending_tx_handler: 0,
            m_psk_context: TEapPskContext::new(),
            data: None,
        }
    }
    pub fn reset (&mut self, ext_addr: TExtendedAddress, short_addr: u16) {
        self.state = DeviceState::BS_STATE_WAITING_JOINNING;
        self.m_lbd_address = ext_addr;
        self.us_lba_src_addr = 0;
        self.us_assigned_short_address = short_addr;
        self.uc_tx_handle = 0xff;
        self.ul_timeout = 0;
        self.uc_tx_attemps = 0;
        self.m_rand_s = TEapPskRand::new();
        self.uc_pending_confirms = 0;
        self.uc_pending_tx_handler = 0;
        self.m_psk_context = TEapPskContext::new();
        self.data = None;

    }
}

type DeviceSlotRef = Rc<RefCell<DeviceSlot>>;

struct DeviceManager {
    devices: HashMap<TExtendedAddress, DeviceSlotRef>,
    short_addresses: HashMap<u16, DeviceSlotRef>,
    // ip_addresses: HashMap<IpAddr, DeviceSlotRef>,
    initial_short_address: u16,
}
impl DeviceManager {
    fn new() -> Self {
        DeviceManager {
            devices: HashMap::new(),
            short_addresses: HashMap::new(),
            // ip_addresses: HashMap::new(),
            initial_short_address: 1,
        }
    }
   
    fn short_addr_from_ip_address(ip_addr: &IpAddr) -> Option<u16> {
        match ip_addr {
            IpAddr::V4(ip) => {
                let octets = ip.octets();
                Some(u16::from_be_bytes([octets[3], octets[4]]))                
            },
            IpAddr::V6(_) => None,
        }
    }
    fn get_device_by_short_addr(&self, short_addr: u16) -> Option<&DeviceSlotRef> {
        self.short_addresses.get(&short_addr)
    }
    fn get_device_by_addr(&self, addr: &TExtendedAddress) -> Option<&DeviceSlotRef> {
        self.devices.get(addr)
    }
    // fn get_device_by_ip(&self, ip: &IpAddr) -> Option<&DeviceSlotRef> {
    //     self.ip_addresses.get(&ip)
    // }
    fn next_short_address(&self) -> u16 {
        let mut short_addr = self.initial_short_address;
        while self.short_addresses.contains_key(&short_addr) {
            short_addr += 1;
        }        
        short_addr
    }
    fn add_or_get_by_addr(&mut self, addr: &TExtendedAddress) -> &DeviceSlotRef {
        let short_addr = self.next_short_address();
        self.devices
            .entry(*addr)
            .or_insert_with(|| {
                self.initial_short_address = short_addr;
                
                let d = Rc::new(RefCell::new(DeviceSlot::new (*addr, short_addr)));
                self.short_addresses.insert(short_addr, d.clone());
                return d;
            })
    }
    fn get_devices(&self) -> &HashMap<TExtendedAddress, DeviceSlotRef> {
        &self.devices
    }
}

pub struct LbpManager {
    u8_eap_identifier: u8,
    ext_addr: [u8; 8],
    pending: u8,
    current_key_index: u8,
    uc_nsdu_handle: u8,
    g_id_s: TEapPskNetworkAccessIdentifierS,
    // devices: HashMap<TExtendedAddress, DeviceSlot>,
    start_time: Instant,
    g_u32_nonce: u32,
    device_manager: DeviceManager,
}

impl LbpManager {
    pub fn new() -> Self {
        let mut id_s: TEapPskNetworkAccessIdentifierS =
            TEapPskNetworkAccessIdentifierS(app_config::X_IDS_CENELEC_FCC.to_vec());
        if app_config::BAND == TAdpBand::ADP_BAND_ARIB {
            id_s = TEapPskNetworkAccessIdentifierS(app_config::X_IDS_ARIB.to_vec());
        }

        LbpManager {
            u8_eap_identifier: 0,
            ext_addr: [0u8; 8],
            pending: 0,
            current_key_index: 0,
            uc_nsdu_handle: 0,
            g_id_s: id_s,
            // devices: HashMap::new(),
            start_time: Instant::now(),
            g_u32_nonce: 0,
            device_manager: DeviceManager::new(),
        }
    }

    fn process_joining_eap_t1(
        p_eap_data: &[u8],
        p_device: &mut DeviceSlot,
        p_id_s: &TEapPskNetworkAccessIdentifierS,
        p_au8_curr_index: u8,
        p_rekey: bool,
        p_u8_eap_identifier: &mut u8,
        p_u32_nonce: &mut u32,
    ) -> Option<Vec<u8>> {
        let mut rand_s: TEapPskRand = TEapPskRand([0; 16]);
        let mut rand_p: TEapPskRand = TEapPskRand([0; 16]);

        let result: Option<Vec<u8>> = None;

        log::trace!("[BS] Process Joining EAP T1");

        if eap_psk_decode_message2(
            p_eap_data,
            &p_device.m_psk_context,
            p_id_s,
            &mut rand_s,
            &mut rand_p,
        ) {
            log::trace!("[BS] Decoded Message2.");

            if rand_s.0 != p_device.m_rand_s.0 {
                log::warn!("[BS] ERROR: Bad RandS received");
                return result;
            }

            eap_psk_initialize_tek(&rand_p, &mut p_device.m_psk_context);

            /* encode and send the message T2 */
            let u16_short_addr = p_device.us_assigned_short_address;
            let mut p_data: Vec<u8> = Vec::new();
            p_data.push(0x02);

            if !p_rekey {
                p_data.push(CONF_PARAM_SHORT_ADDR);

                p_data.push(2);
                p_data.push(((u16_short_addr & 0xFF00) >> 8) as u8);
                p_data.push((u16_short_addr & 0x00FF) as u8);

                p_data.push(CONF_PARAM_GMK);
                p_data.push(17);
                p_data.push(p_au8_curr_index);
                p_data.append(app_config::g_au8CurrGMK.to_vec().borrow_mut());

                p_data.push(CONF_PARAM_GMK_ACTIVATION);
                p_data.push(1);
                p_data.push(p_au8_curr_index);
            } else {
                p_data.push(CONF_PARAM_GMK);
                p_data.push(17);
                p_data.push(p_au8_curr_index ^ 0x01);
                p_data.append(app_config::g_au8RekeyGMK.to_vec().borrow_mut());
            }

            log::trace!("[BS] Encoding Message3.");

            p_device.data = EAP_PSK_Encode_Message3(
                &p_device.m_psk_context,
                *p_u8_eap_identifier,
                &rand_s,
                &rand_p,
                p_id_s,
                *p_u32_nonce,
                PCHANNEL_RESULT_DONE_SUCCESS,
                &p_data,
            );

            *p_u8_eap_identifier += 1;
            *p_u32_nonce += 1;
            let mut v: Option<Vec<u8>> = None;
            if let Some(data) = &p_device.data {
                v = Some(data.to_vec());
            }
            return Some(
                lbp::ChallengeMessage {
                    ext_addr: p_device.m_lbd_address,
                    bootstrapping_data: v, // bootstrapping_data: vec![0x1, 0x2, 0x3]
                }
                .into(),
            );
        } else {
            log::error!("[BS] ERROR: Process_Joining_EAP_T1.");
        }

        // return(uc_result);
        return None;
    }

    fn process_joining_eap_t3(
        p_device: &mut DeviceSlot,
        p_bootstrapping_data: &Vec<u8>,
        p_eap_data: &Vec<u8>,
        p_u8_eap_identifier: &mut u8,
    ) -> Option<Vec<u8>> {
        let mut rand_s: TEapPskRand = TEapPskRand::new();
        let mut u8_pchannel_result: u8 = 0;
        let mut u32_nonce: u32 = 0;
        let mut channel_data: Vec<u8> = Vec::new();
        log::trace!("[BS] Process Joining EAP T3.");

        if EAP_PSK_Decode_Message4(
            p_eap_data,
            &p_device.m_psk_context,
            p_bootstrapping_data,
            &mut rand_s,
            &mut u32_nonce,
            &mut u8_pchannel_result,
            &mut channel_data,
        ) {
            p_device.data = EAP_PSK_Encode_EAP_Success(*p_u8_eap_identifier);
            if rand_s.0 != p_device.m_rand_s.0 {
                log::warn!("[BS] Error: Bad RandS received");
                return None;
            }

            *p_u8_eap_identifier += 1;

            /* Encode now the LBP message */
            return Some(
                lbp::AcceptedMessage {
                    ext_addr: p_device.m_lbd_address,
                    bootstrapping_data: p_device.data.as_ref().map(|v| v.to_vec()),
                }
                .into(),
            );
        } else {
            log::warn!("[BS] ERROR: Process_Joining_EAP_T3.");
            return None;
        }
    }

    fn get_next_short_addr(current: &mut u16) -> u16 {
        *current += 1;
        return *current;
    }
    fn process_joining0(&mut self, msg: &JoiningMessage) -> Option<Vec<u8>> {
        log::trace!("[BS] Process Joining 0.");
        let device = self
            .device_manager
            .add_or_get_by_addr(&msg.ext_addr)
            .deref();
        // .entry(msg.ext_addr)
        // .or_insert(DeviceSlot::new(msg.ext_addr, LbpManager::get_next_short_addr(&mut self.initialShortAddr)));

        if msg.bootstrapping_data.len() == 0 {
            let mut device = device.borrow_mut();
            if device.state != DeviceState::BS_STATE_WAITING_JOINNING {
                let o_short_addr = device.us_assigned_short_address;
                let o_ext_addr = device.m_lbd_address;
                device.reset(o_ext_addr, o_short_addr);
            }
            //First join message
            if (device.state == DeviceState::BS_STATE_WAITING_JOINNING) {
                eap_psk_initialize(&app_config::G_EAP_PSK_KEY, &mut device.m_psk_context);
                device.m_rand_s = TEapPskRand::new_random(); //TODO for testing we need deterministic to compare between the C coordinator and Rust coordinator
                                                            // device.m_randS = config::RAND_S_DEFAULT.to_vec().into();

                device.data = Some(eap_psk_encode_message1(
                    self.u8_eap_identifier,
                    &device.m_rand_s,
                    &self.g_id_s,
                ));
                log::trace!("Process joining message, out_message {:?}", device.data);
                device.state = DeviceState::BS_STATE_SENT_EAP_MSG_1;
                self.u8_eap_identifier += 1;
                return Some(
                    lbp::ChallengeMessage {
                        ext_addr: msg.ext_addr,
                        bootstrapping_data: device.data.as_ref().map(|v| v.to_vec()),
                    }
                    .into(),
                );
            }
        } else {
            let mut pu8_code = 0u8;
            let mut pu8_identifier = 0u8;
            let mut pu8_tsubfield = 0u8;
            let mut p_eap_data: Vec<u8> = Vec::new();
            if eap_psk_decode_message(
                &msg.bootstrapping_data,
                &mut pu8_code,
                &mut pu8_identifier,
                &mut pu8_tsubfield,
                &mut p_eap_data,
            ) {
                if pu8_code == EAP_RESPONSE {
                    let mut device = device.borrow_mut();
                    if pu8_tsubfield == EAP_PSK_T1
                        && (device.state == DeviceState::BS_STATE_WAITING_EAP_MSG_2
                            || device.state == DeviceState::BS_STATE_SENT_EAP_MSG_1)
                    {
                        if let Some(result) = LbpManager::process_joining_eap_t1(
                            &p_eap_data,
                            &mut device,
                            &self.g_id_s,
                            self.current_key_index,
                            false,
                            &mut self.u8_eap_identifier,
                            &mut self.g_u32_nonce,
                        ) {
                            device.state = DeviceState::BS_STATE_SENT_EAP_MSG_3;
                            log::trace!("[BS] Slot updated to BS_STATE_SENT_EAP_MSG_3");
                            return Some(result);
                        } else {
                            /* Abort current BS process */
                            log::trace!("[BS] LBP error processing EAP T1.");
                            device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                            device.uc_pending_confirms = 0;
                            log::trace!("[BS] Slot updated to BS_STATE_WAITING_JOINNING");
                        }
                    } else if pu8_tsubfield == EAP_PSK_T3
                        && (device.state == DeviceState::BS_STATE_WAITING_EAP_MSG_4
                            || device.state == DeviceState::BS_STATE_SENT_EAP_MSG_3)
                    {
                        if let Some(result) = LbpManager::process_joining_eap_t3(
                            &mut device,
                            &msg.bootstrapping_data,
                            &mut p_eap_data,
                            &mut self.u8_eap_identifier,
                        ) {
                            device.state = DeviceState::BS_STATE_SENT_EAP_MSG_ACCEPTED;
                            log::trace!("[BS] Slot updated to BS_STATE_SENT_EAP_MSG_ACCEPTED");
                            return Some(result);
                        } else {
                            log::warn!("[BS] LBP error processing EAP T3.");
                            device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                            device.uc_pending_confirms = 0;
                            log::trace!("[BS] Slot updated to BS_STATE_WAITING_JOINNING");
                        }
                    } else {
                        /* Abort current BS process */

                        log::warn!("[BS] protocol error. from device {:?}", device);
                        device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                        device.uc_pending_confirms = 0;
                        log::trace!(
                            "[BS] Slot updated to BS_STATE_WAITING_JOINNING device {:?}",
                            device
                        );
                    }
                }
            } else {
                log::warn!("[BS] ERROR decoding message. from device {:?}", device);

                // device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                // device.uc_pending_confirms = 0;
                // log::trace!(
                //     "[BS] Slot updated to BS_STATE_WAITING_JOINNING device {:?}",
                //     device
                // );
            }
        }

        None

        // p_bs_slot->us_data_length = LBP_Encode_ChallengeRequest(
        // 		&pLBPEUI64Address,
        // 		p_bs_slot->us_data_length,
        // 		u16MemoryBufferLength,
        // 		pMemoryBuffer
        // 		);

        // if (!m_bRekey) {
        // 	/* If extended address is already in list, remove it and give a new short address */
        // 	if (bs_get_short_addr_by_ext(pLBPEUI64Address.m_au8Value, &u16DummyShortAddress)) {
        // 		remove_lbds_list_entry(u16DummyShortAddress);
        // 	}

        // 	/* Get a new address for the device. Its extended address will be added to the list when the joining process finishes. */
        // 	p_bs_slot->us_assigned_short_address = get_new_address(p_bs_slot->m_LbdAddress);
        // }
    }

    pub fn process_response(&mut self, lbp_response: &AdpG3LbpReponse) {
        for (_addr, device) in self.device_manager.get_devices() {
            if device.borrow().uc_pending_confirms == 1
                && lbp_response.handle == device.borrow().uc_tx_handle
                && device.borrow().state != DeviceState::BS_STATE_WAITING_JOINNING
            {
                let mut device = device.deref().borrow_mut();
                device.uc_pending_confirms -= 1;

                if lbp_response.status == EAdpStatus::G3_SUCCESS {
                    if device.uc_pending_confirms == 0 {
                        match device.state {
                            DeviceState::BS_STATE_SENT_EAP_MSG_1 => {
                                device.state = DeviceState::BS_STATE_WAITING_EAP_MSG_2;
                            }
                            DeviceState::BS_STATE_SENT_EAP_MSG_3 => {
                                device.state = DeviceState::BS_STATE_WAITING_EAP_MSG_4;
                            }
                            DeviceState::BS_STATE_SENT_EAP_MSG_ACCEPTED => {
                                device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                                device.uc_pending_confirms = 0;
                            }
                            DeviceState::BS_STATE_SENT_EAP_MSG_DECLINED => {
                                device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                                device.uc_pending_confirms = 0;
                            }
                            _ => {
                                device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                                device.uc_pending_confirms = 0;
                            }
                        }
                        log::trace!("device log {:?}", device);
                    }
                } else {
                    device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                    device.uc_pending_confirms = 0;
                }
                device.ul_timeout = self.start_time.elapsed().as_millis() + UC_MESSAGE_TIMEOUT_MS;
            } else if (device.borrow().uc_pending_confirms == 2
                && lbp_response.handle == device.borrow().uc_pending_tx_handler)
            {
                let mut device = device.deref().borrow_mut();
                device.uc_pending_confirms -= 1;
                device.ul_timeout = self.start_time.elapsed().as_millis() + UC_MESSAGE_TIMEOUT_MS;
            }
        }
    }
    pub fn process_msg(&mut self, lbp_message: &lbp::LbpMessage) -> Option<request::AdpLbpRequest> {
        let mut out_message: Option<Vec<u8>> = None;
        let mut addr: Option<TExtendedAddress> = None;

        match lbp_message {
            lbp::LbpMessage::Joining(joining_message) => {
                addr = Some(joining_message.ext_addr);
                out_message = self.process_joining0(&joining_message);
            }
            lbp::LbpMessage::Accepted(_) => todo!(),
            lbp::LbpMessage::Challenge(_) => todo!(),
            lbp::LbpMessage::Decline(_) => todo!(),
            lbp::LbpMessage::KickFromLbd(_) => todo!(),
            lbp::LbpMessage::KickToLbd(_) => todo!(),
        }
        if let (Some(out), Some(addr)) = (out_message, addr) {
            if let Some(device) = self.device_manager.get_device_by_addr(&addr) {
                let mut device = device.deref().borrow_mut();
                if (device.uc_pending_confirms > 0) {
                    device.uc_pending_tx_handler = device.uc_tx_handle;
                }
                self.uc_nsdu_handle += 1;
                device.uc_tx_handle = self.uc_nsdu_handle;
                device.ul_timeout = self.start_time.elapsed().as_millis() + UC_MESSAGE_TIMEOUT_MS;
                device.uc_tx_attemps = 0;
                device.uc_pending_confirms += 1;
                return Some(request::AdpLbpRequest::new(
                    addr.into(),
                    out,
                    device.uc_tx_handle - 1,
                    app_config::MAX_HOPS,
                    true,
                    0,
                    false,
                ));
            }
        }

        None
    }

    // pub fn update_devices(&mut self)
    // {
    //     uint8_t uc_i;

    //     for (uc_i = 0; uc_i < BOOTSTRAP_NUM_SLOTS; uc_i++) {
    //         /* log_show_slots_status(); */
    //         if (bootstrap_slots[uc_i].e_state != BS_STATE_WAITING_JOINNING) {
    //             if (timeout_is_past(bootstrap_slots[uc_i].ul_timeout)) {
    //                 LOG_BOOTSTRAP(("[BS] timeout_is_past for %d\r\n",uc_i));
    //                 if (bootstrap_slots[uc_i].uc_pending_confirms == 0) {
    //                     if (bootstrap_slots[uc_i].uc_tx_attemps < BOOTSTRAP_MSG_MAX_RETRIES) {
    //                         bootstrap_slots[uc_i].uc_tx_attemps++;
    //                         if (bootstrap_slots[uc_i].e_state == BS_STATE_WAITING_EAP_MSG_2) {
    //                             bootstrap_slots[uc_i].e_state = BS_STATE_SENT_EAP_MSG_1;
    //                             LOG_BOOTSTRAP(("[BS] Slot updated to BS_STATE_SENT_EAP_MSG_1\r\n"));
    //                             log_show_slots_status();
    //                         } else if (bootstrap_slots[uc_i].e_state == BS_STATE_WAITING_EAP_MSG_4) {
    //                             bootstrap_slots[uc_i].e_state = BS_STATE_SENT_EAP_MSG_3;
    //                             LOG_BOOTSTRAP(("[BS] Slot updated to BS_STATE_SENT_EAP_MSG_3\r\n"));
    //                             log_show_slots_status();
    //                         }

    //                         struct TAddress dstAddr;
    //                         struct TAdpGetConfirm getConfirm;

    //                         if (bootstrap_slots[uc_i].us_data_length > 0) {
    //                             if (bootstrap_slots[uc_i].us_lba_src_addr == 0xFFFF) {
    //                                 dstAddr.m_u8AddrLength = 8;
    //                                 memcpy(dstAddr.m_u8ExtendedAddr, &bootstrap_slots[uc_i].m_LbdAddress.m_au8Value, 8);
    //                             } else {
    //                                 dstAddr.m_u8AddrLength = 2;
    //                                 dstAddr.m_u16ShortAddr = bootstrap_slots[uc_i].us_lba_src_addr;
    //                             }

    //                             if (bootstrap_slots[uc_i].uc_pending_confirms > 0) {
    //                                 bootstrap_slots[uc_i].uc_pending_tx_handler = bootstrap_slots[uc_i].uc_tx_handle;
    //                             }

    //                             bootstrap_slots[uc_i].uc_tx_handle = get_next_nsdu_handler();
    //                             bootstrap_slots[uc_i].ul_timeout = oss_get_up_time_ms() + 1000 * us_msg_timeout_in_s;
    //                             bootstrap_slots[uc_i].uc_pending_confirms++;

    //                             LOG_BOOTSTRAP(("[BS] Timeout detected. Re-sending MSG for slot: %d Attempt: %d \r\n", uc_i,
    //                                     bootstrap_slots[uc_i].uc_tx_attemps));
    //                             log_show_slots_status();
    //                             LOG_BOOTSTRAP(("[BS] AdpLbpRequest Called, handler: %d \r\n", bootstrap_slots[uc_i].uc_tx_handle));
    //                             AdpLbpRequest((struct TAdpAddress const *)&dstAddr,     /* Destination address */
    //                                     bootstrap_slots[uc_i].us_data_length,                              /* NSDU length */
    //                                     &bootstrap_slots[uc_i].auc_data[0],                                  /* NSDU */
    //                                     bootstrap_slots[uc_i].uc_tx_handle,                            /* NSDU handle */
    //                                     g_s_bs_conf.m_u8MaxHop,          						/* Max. Hops */
    //                                     true,                                       /* Discover route */
    //                                     0,                                          /* QoS */
    //                                     false);                                     /* Security enable */
    //                         }
    //                     } else {
    //                         LOG_BOOTSTRAP(("[BS] Reset slot %d:  \r\n", uc_i));
    //                         bootstrap_slots[uc_i].e_state = BS_STATE_WAITING_JOINNING;
    //                         bootstrap_slots[uc_i].uc_pending_confirms = 0;
    //                         bootstrap_slots[uc_i].ul_timeout = 0xFFFFFFFF;
    //                     }
    //                 } else { /* Pending confirm then increase timeout time */
    //                     LOG_BOOTSTRAP(("[BS] Pending confirm\r\n"));
    //                     bootstrap_slots[uc_i].ul_timeout = oss_get_up_time_ms() + 1000 * us_msg_timeout_in_s;
    //                 }
    //             }
    //         }
    //     }
    // }
}

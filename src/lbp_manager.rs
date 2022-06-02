use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::hash::Hash;
use std::process::id;
use std::time::Instant;
use std::time::SystemTime;

use crate::adp::TAdpBand;
use crate::adp::TExtendedAddress;
use crate::config;
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
    m_LbdAddress: TExtendedAddress,
    us_lba_src_addr: u16,
    us_assigned_short_address: u16,
    uc_tx_handle: u8,
    ul_timeout: u128,
    uc_tx_attemps: u8,
    m_randS: TEapPskRand,
    uc_pending_confirms: u8,
    uc_pending_tx_handler: u8,
    m_PskContext: TEapPskContext,
}
impl DeviceSlot {
    pub fn new(ext_addr: TExtendedAddress) -> Self {
        DeviceSlot {
            state: DeviceState::BS_STATE_WAITING_JOINNING,
            m_LbdAddress: ext_addr,
            us_lba_src_addr: 0,
            us_assigned_short_address: 0,
            uc_tx_handle: 0,
            ul_timeout: 0,
            uc_tx_attemps: 0,
            m_randS: TEapPskRand::new(),
            uc_pending_confirms: 0,
            uc_pending_tx_handler: 0,
            m_PskContext: TEapPskContext::new(),
        }
    }
}

pub struct LbpManager {
    u8EAPIdentifier: u8,
    initialShortAddr: u16,
    extAddr: [u8; 8],
    pending: u8,
    currentKeyIndex: u8,
    uc_nsdu_handle: u8,
    g_IdS: TEapPskNetworkAccessIdentifierS,
    devices: HashMap<TExtendedAddress, DeviceSlot>,
    start_time: Instant,
    g_u32Nonce: u32,
}

impl LbpManager {
    pub fn new() -> Self {
        let mut idS: TEapPskNetworkAccessIdentifierS =
            TEapPskNetworkAccessIdentifierS(config::X_IDS_CENELEC_FCC.to_vec());
        if config::BAND == TAdpBand::ADP_BAND_ARIB {
            idS = TEapPskNetworkAccessIdentifierS(config::X_IDS_ARIB.to_vec());
        }

        LbpManager {
            u8EAPIdentifier: 0,
            initialShortAddr: 0,
            extAddr: [0u8; 8],
            pending: 0,
            currentKeyIndex: 0,
            uc_nsdu_handle: 0,
            g_IdS: idS,
            devices: HashMap::new(),
            start_time: Instant::now(),
            g_u32Nonce: 0,
        }
    }

    fn Process_Joining_EAP_T1(
        pEAPData: &Vec<u8>,
        pDevice: &mut DeviceSlot,
        pIds: &TEapPskNetworkAccessIdentifierS,
        p_au8CurrIndex: u8,
        pRekey: bool,
        p_u8EAPIdentifier: &mut u8,
        p_u32Nonce: &mut u32,
    ) -> Option<Vec<u8>> {
        let mut randS: TEapPskRand = TEapPskRand([0; 16]);
        let mut randP: TEapPskRand = TEapPskRand([0; 16]);

        let mut result: Option<Vec<u8>> = None;

        log::trace!("[BS] Process Joining EAP T1");

        if EAP_PSK_Decode_Message2(
            pEAPData,
            &pDevice.m_PskContext,
            pIds,
            &mut randS,
            &mut randP,
        ) {
            log::trace!("[BS] Decoded Message2.");

            if randS.0 != pDevice.m_randS.0 {
                log::warn!("[BS] ERROR: Bad RandS received");
                return result;
            }

            EAP_PSK_InitializeTEK(&randP, &mut pDevice.m_PskContext);

            /* encode and send the message T2 */
            let u16ShortAddr = pDevice.us_assigned_short_address;
            let mut pData: Vec<u8> = Vec::new();
            pData.push(0x02);

            if !pRekey {
                pData.push(CONF_PARAM_SHORT_ADDR);

                pData.push(2);
                pData.push(((u16ShortAddr & 0xFF00) >> 8) as u8);
                pData.push((u16ShortAddr & 0x00FF) as u8);

                pData.push(CONF_PARAM_GMK);
                pData.push(17);
                pData.push(p_au8CurrIndex);
                pData.append(config::g_au8CurrGMK.to_vec().borrow_mut());

                pData.push(CONF_PARAM_GMK_ACTIVATION);
                pData.push(1);
                pData.push(p_au8CurrIndex);
            } else {
                // /* Get current key index and set the new one to the other */
                // if (p_au8CurrIndex == 0) {
                // 	u8NewKeyIndex = 1;
                // } else {
                // 	u8NewKeyIndex = 0;
                // }
                // replace with u8NewKeyIndex = p_au8CurrIndex ^ 1;
                pData.push(CONF_PARAM_GMK);
                pData.push(17);
                pData.push(p_au8CurrIndex ^ 0x01);
                pData.append(config::g_au8RekeyGMK.to_vec().borrow_mut());
            }

            log::trace!("[BS] Encoding Message3.");
            let mut pMemoryBuffer: Vec<u8> = Vec::new();
            EAP_PSK_Encode_Message3(
                &pDevice.m_PskContext,
                *p_u8EAPIdentifier,
                &randS,
                &randP,
                pIds,
                *p_u32Nonce,
                PCHANNEL_RESULT_DONE_SUCCESS,
                &pData,
                &mut pMemoryBuffer,
            );

            *p_u8EAPIdentifier += 1;
            *p_u32Nonce += 1;

            return Some(
                lbp::ChallengeMessage {
                    ext_addr: pDevice.m_LbdAddress,
                    bootstrapping_data: pMemoryBuffer,
                }
                .into(),
            );
        } else {
            log::error!("[BS] ERROR: Process_Joining_EAP_T1.");
        }

        // return(uc_result);
        return None;
    }

    fn Process_Joining_EAP_T3(pDevice: &mut DeviceSlot, pBootstrappingData: &Vec<u8>,
		pEAPData: &Vec<u8>, p_u8EAPIdentifier: &mut u8) -> Option<Vec<u8>>
{
	let mut randS: TEapPskRand = TEapPskRand::new();
	let mut u8PChannelResult:u8 = 0;
	let mut u32Nonce:u32 = 0;
    let mut channelData:Vec<u8> = Vec::new();
	log::trace!("[BS] Process Joining EAP T3.");

	if (EAP_PSK_Decode_Message4(pEAPData, &pDevice.m_PskContext,
			pBootstrappingData, &mut randS, &mut u32Nonce, &mut u8PChannelResult,
			&mut channelData)) {
        let mut memory_buffer: Vec<u8> = Vec::new();
		EAP_PSK_Encode_EAP_Success(
				*p_u8EAPIdentifier,
				&mut memory_buffer
				);
        if randS.0 != pDevice.m_randS.0 {
            log::warn!("[BS] Error: Bad RandS received");
            return None;
        }

		*p_u8EAPIdentifier +=1;

		/* Encode now the LBP message */

                return Some(
                    lbp::AcceptedMessage {
                        ext_addr: pDevice.m_LbdAddress,
                        bootstrapping_data: memory_buffer,
                    }
                    .into(),
                );        

	} else {
		log::warn!("[BS] ERROR: Process_Joining_EAP_T3.");
		return None;
	}
}

    fn Process_Joining0(&mut self, msg: &JoiningMessage) -> Option<Vec<u8>> {
        log::trace!("[BS] Process Joining 0.");
        let device = self
            .devices
            .entry(msg.ext_addr)
            .or_insert(DeviceSlot::new(msg.ext_addr));

        if (msg.bootstrapping_data.len() == 0) {
            //First join message
            if (device.state == DeviceState::BS_STATE_WAITING_JOINNING) {
                EAP_PSK_Initialize(&config::G_EAP_PSK_KEY, &mut device.m_PskContext);
                device.m_randS = TEapPskRand::new_random();

                let mut out_message: Vec<u8> = Vec::new();

                EAP_PSK_Encode_Message1(
                    self.u8EAPIdentifier,
                    &device.m_randS,
                    &self.g_IdS,
                    &mut out_message,
                );
                device.state = DeviceState::BS_STATE_SENT_EAP_MSG_1;
                self.u8EAPIdentifier += 1;
                return Some(
                    lbp::ChallengeMessage {
                        ext_addr: msg.ext_addr,
                        bootstrapping_data: out_message,
                    }
                    .into(),
                );
            }
        } else {
            let mut pu8Code = 0u8;
            let mut pu8Identifier = 0u8;
            let mut pu8TSubfield = 0u8;
            let mut pEAPData: Vec<u8> = Vec::new();
            if EAP_PSK_Decode_Message(
                &msg.bootstrapping_data,
                &mut pu8Code,
                &mut pu8Identifier,
                &mut pu8TSubfield,
                &mut pEAPData,
            ) {
                if pu8Code == EAP_RESPONSE {
                    if (pu8Identifier == EAP_PSK_T1
                        && (device.state == DeviceState::BS_STATE_WAITING_EAP_MSG_2
                            || device.state == DeviceState::BS_STATE_SENT_EAP_MSG_1))
                    {
                        if let Some(result) = LbpManager::Process_Joining_EAP_T1(
                            &pEAPData,
                            device,
                            &self.g_IdS,
                            self.currentKeyIndex,
                            false, &mut self.u8EAPIdentifier, &mut self.g_u32Nonce
                        ) {
                            /* Abort current BS process */
                            log::trace!("[BS] LBP error processing EAP T1.");
                            device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                            device.uc_pending_confirms = 0;
                            log::trace!("[BS] Slot updated to BS_STATE_WAITING_JOINNING");
                            return Some(result);
                        } else {
                            device.state = DeviceState::BS_STATE_SENT_EAP_MSG_3;
                            log::trace!("[BS] Slot updated to BS_STATE_SENT_EAP_MSG_3");
                        }
                    } else if pu8TSubfield == EAP_PSK_T3
                        && (device.state == DeviceState::BS_STATE_WAITING_EAP_MSG_4
                            || device.state == DeviceState::BS_STATE_SENT_EAP_MSG_3)
                    {
                        if let Some(result) = LbpManager::Process_Joining_EAP_T3(device, &msg.bootstrapping_data, &mut pEAPData,
                                &mut self.u8EAPIdentifier) {
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
                device.state = DeviceState::BS_STATE_WAITING_JOINNING;
                device.uc_pending_confirms = 0;
                log::trace!(
                    "[BS] Slot updated to BS_STATE_WAITING_JOINNING device {:?}",
                    device
                );
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

    pub fn process_msg(&mut self, lpb_message: &lbp::LbpMessage) -> Option<request::AdpLbpRequest> {
        let mut out_message: Option<Vec<u8>> = None;
        let addr: Option<TExtendedAddress> = None;

        match lpb_message {
            lbp::LbpMessage::Joining(joining_message) => {
                out_message = self.Process_Joining0(&joining_message);
            }
            lbp::LbpMessage::Accepted(_) => todo!(),
            lbp::LbpMessage::Challenge(_) => todo!(),
            lbp::LbpMessage::Decline(_) => todo!(),
            lbp::LbpMessage::KickFromLbd(_) => todo!(),
            lbp::LbpMessage::KickToLbd(_) => todo!(),
        }
        if let (Some(out), Some(addr)) = (out_message, addr) {
            if let Some(device) = self.devices.get_mut(&addr) {
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
                    device.uc_tx_handle,
                    config::MAX_HOPS,
                    true,
                    0,
                    false,
                ));
            }
        }

        None
    }
}

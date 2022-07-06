use std::borrow::BorrowMut;


use aes::Aes128;
use aes::NewBlockCipher;
use cmac::{Cmac, Mac, NewMac};
use rand::Rng;
use rand::thread_rng;
use std::fmt::Debug;

use eax::aead::{AeadMut, Payload};
use eax::aead::generic_array::GenericArray;

use eax::{Eax, NewAead};



pub const EAP_PSK_IANA_TYPE: u8 = 0x2F;
pub const KEY_LEN: usize = 16;

/**********************************************************************************************************************/
/** EAP message types
 *
 ***********************************************************************************************************************
 *
 * The value takes in account the 2 reserved bits (values are left shifted by 2 bits)
 *
 **********************************************************************************************************************/
pub const EAP_REQUEST: u8 = 0x04;
pub const EAP_RESPONSE: u8 = 0x08;
pub const EAP_SUCCESS: u8 = 0x0C;
pub const EAP_FAILURE: u8 = 0x10;

/**********************************************************************************************************************/
/** T-subfield types
 *
 ***********************************************************************************************************************
 *
 * 0 The first EAP-PSK message
 * 1 The second EAP-PSK message
 * 2 The third EAP-PSK message
 * 3 The forth EAP-PSK message
 *
 **********************************************************************************************************************/
pub const EAP_PSK_T0: u8 = (0x00 << 6);
pub const EAP_PSK_T1: u8 = (0x01 << 6);
pub const EAP_PSK_T2: u8 = (0x02 << 6);
pub const EAP_PSK_T3: u8 = (0x03 << 6);

/**********************************************************************************************************************/
/** P-Channel result field
 *
 ***********************************************************************************************************************
 *
 **********************************************************************************************************************/
pub const PCHANNEL_RESULT_CONTINUE: u8 = 0x01;
pub const PCHANNEL_RESULT_DONE_SUCCESS: u8 = 0x02;
pub const PCHANNEL_RESULT_DONE_FAILURE: u8 = 0x03;

/**********************************************************************************************************************/
/** The EAP_PSK NetworkAccessIdentifier P & S types
 ***********************************************************************************************************************
 *
 **********************************************************************************************************************/
pub const NETWORK_ACCESS_IDENTIFIER_MAX_SIZE_S: usize = 34;
pub const NETWORK_ACCESS_IDENTIFIER_MAX_SIZE_P: usize = 36;

pub const NETWORK_ACCESS_IDENTIFIER_SIZE_S_ARIB: usize = NETWORK_ACCESS_IDENTIFIER_MAX_SIZE_S;
pub const NETWORK_ACCESS_IDENTIFIER_SIZE_P_ARIB: usize = NETWORK_ACCESS_IDENTIFIER_MAX_SIZE_P;

pub const NETWORK_ACCESS_IDENTIFIER_SIZE_S_CENELEC_FCC: usize = 8;
pub const NETWORK_ACCESS_IDENTIFIER_SIZE_P_CENELEC_FCC: usize = 8;

pub struct TEapPskNetworkAccessIdentifierP(pub Vec<u8>);

pub struct TEapPskNetworkAccessIdentifierS(pub Vec<u8>);

/**********************************************************************************************************************/
/** The EAP_PSK key type
 ***********************************************************************************************************************
 *
 **********************************************************************************************************************/
#[derive(Clone)]
pub struct TEapPskKey(pub [u8; 16]);
impl TEapPskKey {}
impl From<Vec<u8>> for TEapPskKey {
    fn from(v: Vec<u8>) -> Self {
        let u: [u8; 16] = v.try_into().unwrap();

        return TEapPskKey(u);
    }
}
impl Debug for TEapPskKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TEapPskKey").field(&format_args!("{:x?}", self.0)).finish()
    }
}


/**********************************************************************************************************************/
/** The EAP_PSK RAND type
 ***********************************************************************************************************************
 *
 **********************************************************************************************************************/
pub struct TEapPskRand(pub [u8; 16]);
impl TEapPskRand {
    pub fn new() -> Self{
        TEapPskRand([0u8; 16])
    }
    pub fn new_random() -> Self {
        let mut arr = [0u8; 16];
        thread_rng().fill(&mut arr);
        TEapPskRand(arr)
    }
}
impl From<Vec<u8>> for TEapPskRand {
    fn from(v: Vec<u8>) -> Self {
        let u: [u8; 16] = v.try_into().unwrap(); //TODO, remove the unwrap

        return TEapPskRand(u);
    }
}
impl From<&[u8]> for TEapPskRand {
    fn from(v: &[u8]) -> Self {
        let u: [u8; 16] = v.try_into().unwrap(); //TODO, remove the unwrap

        return TEapPskRand(u);
    }
}

impl Debug for TEapPskRand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TEapPskRand").field(&format_args!("{:x?}", self.0)).finish()
    }
}
// impl Debug for TEapPskRand {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("TEapPskRand")
//             // ...
//             .field("SP", &format_args!("{:x?}", self.0))            
//             // ...
//             .finish()
//     }
// }
/**********************************************************************************************************************/
/** The EAP_PSK NetworkAccessIdentifier
 ***********************************************************************************************************************
 *
 **********************************************************************************************************************/
#[derive(Debug)]
pub struct TEapPskNetworkAccessIdentifier(Vec<u8>);
impl TEapPskNetworkAccessIdentifier {
    pub fn new() -> Self {
        TEapPskNetworkAccessIdentifier(Vec::new())
    }
}

/**********************************************************************************************************************/
/** The EAP_PSK_Context type keeps information needed for EAP-PSK calls
 ***********************************************************************************************************************
 *
 **********************************************************************************************************************/
#[derive(Debug)]
pub struct TEapPskContext {
    m_Kdk: TEapPskKey, // Derivation key
    m_Ak: TEapPskKey,  // Authentication key
    m_Tek: TEapPskKey, // Transient key
    m_IdS: TEapPskNetworkAccessIdentifier,
    m_RandP: TEapPskRand,
    m_RandS: TEapPskRand,
}
impl TEapPskContext {
    pub fn new() -> Self {
        TEapPskContext {
            m_Kdk: TEapPskKey([0; 16]),
            m_Ak: TEapPskKey([0; 16]),
            m_Tek: TEapPskKey([0; 16]),
            m_IdS: TEapPskNetworkAccessIdentifier::new(),
            m_RandP: TEapPskRand([0; 16]),
            m_RandS: TEapPskRand([0; 16]),
        }
    }
}

pub fn eap_psk_initialize(pKey: &TEapPskKey, pPskContext: &mut TEapPskContext) -> bool {
    let mut block = aes::cipher::generic_array::GenericArray::from([0u8; 16]);

    // let encryptor = aes_embedded::AES128::new (&(pKey.0));
    let encryptor = aes::Aes128::new_from_slice(&pKey.0);

    // let encryptor = aes::Aes128::new(aes::cipher::generic_array::GenericArray::from(pKey.0));
    if let Ok(encryptor) = encryptor {
        aes::cipher::BlockEncrypt::encrypt_block(&encryptor, &mut block);
        block[15] ^= 0x01; // xor with c1 = "1"
        let mut ak = block.clone();
        aes::cipher::BlockEncrypt::encrypt_block(&encryptor, &mut ak);
        pPskContext.m_Ak.0 = ak.into();

        // xor with c1 = "2"
        block[15] ^= 0x03; // 3 instead of 2 because it has been already xor'ed with 1 and we want to get back the initial value
        let mut kdk = block.clone();
        aes::cipher::BlockEncrypt::encrypt_block(&encryptor, &mut kdk);
        pPskContext.m_Kdk.0 = kdk.into();
        true
    } else {
        false
    }

    // xor with c1 = "1"
}


pub fn eap_psk_initialize_tek(p_rand_p: &TEapPskRand, p_psk_context: &mut TEapPskContext) -> bool {
    log::info!("->EAP_PSK_InitializeTEK : {:?}, {:?}", p_rand_p, p_psk_context.m_Kdk.0);
    let encryptor = aes::Aes128::new_from_slice(&p_psk_context.m_Kdk.0);
    if let Ok(encryptor) = encryptor {
        let mut v = aes::cipher::generic_array::GenericArray::from(p_rand_p.0);
        aes::cipher::BlockEncrypt::encrypt_block(&encryptor, &mut v);
        v[15] ^= 0x01;
        aes::cipher::BlockEncrypt::encrypt_block(&encryptor, &mut v);
        p_psk_context.m_Tek.0 = v.into();
        log::info!("->EAP_PSK_InitializeTEK : {:?}, {:?}", p_rand_p, p_psk_context.m_Tek);
        true
    } else {
        log::warn!("EAP_PSK_InitializeTEK : Failed to get encryptor");
        false
    }
}

pub fn eap_psk_decode_message(
    p_message: &Vec<u8>,
    pu8_code: &mut u8,
    pu8_identifier: &mut u8,
    pu8_tsubfield: &mut u8,
    p_eap_data: &mut Vec<u8>,
) -> bool {
    let mut b_ret = false;

    if p_message.len() >= 4 {
        *pu8_code = p_message[0];
        *pu8_identifier = p_message[1];
        let u16_eap_message_length: usize = ((p_message[2] as usize) << 8) | p_message[3] as usize;

        // A message with the Length field set to a value larger than the number of received octets MUST be silently discarded.
        b_ret = u16_eap_message_length <= p_message.len();

        if b_ret && (u16_eap_message_length >= 6) {
            *pu8_tsubfield = p_message[5];
            *p_eap_data = p_message[6..].to_vec();
            b_ret = (p_message[4] == EAP_PSK_IANA_TYPE);
        }
    }

    return b_ret;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn eap_psk_decode_message1(
    p_message: &Vec<u8>,
    p_rand_s: &mut TEapPskRand,
    p_id_s: &mut TEapPskNetworkAccessIdentifier,
) -> bool {
    let mut bRet = false;

    // check the length of the message
    if (p_message.len() >= p_rand_s.0.len()) {
        *p_rand_s = p_message[0..16].to_vec().into();
        //   memcpy(pRandS->m_au8Value, pMessage, sizeof(pRandS->m_au8Value));
        p_id_s.0 = p_message[16..].to_vec();
        bRet = true;
    }

    return bRet;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn eap_psk_encode_message2(
    p_psk_context: &TEapPskContext,
    p_u8_identifier: u8,
    p_rand_s: &TEapPskRand,
    p_rand_p: &TEapPskRand,
    p_id_s: &TEapPskNetworkAccessIdentifier,
    p_id_p: &TEapPskNetworkAccessIdentifier,
    p_memory_buffer: &mut Vec<u8>,
) {
    // check the size of the buffer
    if (p_memory_buffer.capacity() >= 62) {
        //TODO panic or increase size?
        // compute first MacP = CMAC-AES-128(AK, IdP||IdS||RandS||RandP)
        let mut au8Seed: Vec<u8> = Vec::new();

        let mut mac = Cmac::<aes::Aes128>::new_from_slice(&p_psk_context.m_Ak.0).unwrap();

        //   ret = cipher_wrapper_cipher_setup( &m_ctx, cipher_info );
        //   LOG_DBG(Log("\n cipher_wrapper_cipher_setup returned %d %d", ret, m_ctx.cipher_info->type));
        au8Seed.append(&mut p_id_p.0.to_vec());
        au8Seed.append(&mut p_id_s.0.to_vec());
        au8Seed.append(&mut p_rand_s.0.to_vec());
        au8Seed.append(&mut p_rand_p.0.to_vec());
        mac.update(&au8Seed);

        let au8_mac_p = mac.finalize();
        // encode the EAP header; length field will be set at the end of the block
        p_memory_buffer.push(EAP_RESPONSE);
        p_memory_buffer.push(p_u8_identifier);
        p_memory_buffer.push(0);
        p_memory_buffer.push(0);
        p_memory_buffer.push(EAP_PSK_IANA_TYPE);
        p_memory_buffer.push(EAP_PSK_T1);
        p_memory_buffer.extend_from_slice(&p_rand_s.0);
        p_memory_buffer.extend_from_slice(&p_rand_p.0);
        p_memory_buffer.extend_from_slice(&au8_mac_p.into_bytes().to_vec());
        p_memory_buffer.extend_from_slice(&p_id_p.0);

        //   // now update the EAP header length field
        p_memory_buffer[2] = ((p_memory_buffer.len() >> 8) & 0x00FF) as u8;
        p_memory_buffer[3] = (p_memory_buffer.len() & 0x00FF) as u8;

        //   UNUSED(ret);
    }
}

/**********************************************************************************************************************/
/** The EAP_PSK_Decode_Message3 primitive is used to decode the third EAP-PSK message (type 2)
 **********************************************************************************************************************/
pub fn eap_psk_decode_message3(
    p_message: &Vec<u8>,
    p_psk_context: &TEapPskContext,
    p_header: Vec<u8>,
    p_rand_s: &mut TEapPskRand,
    p_u32_nonce: &mut u32,
    p_u8_pchannel_result: &mut u8,
    p_pchannel_data: &mut Vec<u8>,
) -> bool {
    let mut b_ret = false;

    if p_message.len() >= 59 {
        *p_rand_s = p_message[0..16].to_vec().into();
        let mac_s = &p_message[16..32];

        // verify MacS: MAC_S = CMAC-AES-128(AK, IdS||RandP)
        let mut au8_seed: Vec<u8> = Vec::new();

        let mut mac = Cmac::<aes::Aes128>::new_from_slice(&p_psk_context.m_Ak.0).unwrap();

        au8_seed.extend_from_slice(&p_psk_context.m_IdS.0);
        au8_seed.extend_from_slice(&p_psk_context.m_RandP.0);
        mac.update(&au8_seed);
        let au8_mac_s = mac.finalize().into_bytes().to_vec();

        if au8_mac_s == mac_s {
            let key = eax::aead::generic_array::GenericArray::from_slice(&p_psk_context.m_Tek.0);
            let p_nonce = &p_message[32..36];
            // let pTag = pMessage[36..52];
            let p_protected_data = &p_message[36..];
            let mut au8_nonce: [u8; 16] = [0; 16];
            au8_nonce[12] = p_nonce[0];
            au8_nonce[13] = p_nonce[1];
            au8_nonce[14] = p_nonce[2];
            au8_nonce[15] = p_nonce[3];
            let mut cipher = eax::Eax::<Aes128>::new(key);
            // The protected data is the 22 bytes header of the EAP message.
            // The G3 specifies a slightly modified EAP header but in the same time
            // the specification requires to compute the authentication tag over the
            // on the original EAP header
            // So we change the header to make it "EAP compliant", we compute the
            // auth tag and then we change back the header

            // right shift Code field with 2 bits as indicated in the EAP specification
            let mut header = p_header.clone();
            header[0] >>= 2; //TODO, check if header has any data

            if let Ok(data) = cipher.decrypt(
                GenericArray::from_slice(&au8_nonce),
                Payload {
                    msg: p_protected_data,
                    aad: &header,
                },
            ) {
                *p_u8_pchannel_result = (data[0] & 0xC0) >> 6;
                *p_pchannel_data = data[1..].to_vec();
                *p_u32_nonce = u32::from_be_bytes([p_nonce[3], p_nonce[2], p_nonce[1], p_nonce[0]]);
            }
        }
    }
    return b_ret;
}

/**********************************************************************************************************************/
/** The EAP_PSK_Encode_Message4 primitive is used to encode the second EAP-PSK message (type 3)
 **********************************************************************************************************************/
pub fn eap_psk_encode_message4(
    p_psk_context: &TEapPskContext,
    u8_identifier: u8,
    p_rand_s: &TEapPskRand,
    u32_nonce: u32,
    u8_pchannel_result: u8,
    p_pchannel_data: Vec<u8>,
    p_protected_data: &mut Vec<u8>,
) -> bool {
    let mut header: Vec<u8> = Vec::with_capacity(22);

    let mut au8_nonce = [0u8; 16];

    // encode the EAP header; length field will be set at the end of the block
    header.push(EAP_RESPONSE);
    header.push(u8_identifier);
    header.push(0);
    header.push(0);
    header.push(EAP_PSK_IANA_TYPE);
    header.push(EAP_PSK_T3);

    // EAP message: add PSK fields

    header.extend_from_slice(&p_rand_s.0);

    // nonce should be big endian
    au8_nonce[12] = ((u32_nonce >> 24) & 0xFF) as u8;
    au8_nonce[13] = ((u32_nonce >> 16) & 0xFF) as u8;
    au8_nonce[14] = ((u32_nonce >> 8) & 0xFF) as u8;
    au8_nonce[15] = ((u32_nonce) & 0xFF) as u8;

    // protected data
    let mut protected_data: Vec<u8> = Vec::new();

    if (p_pchannel_data.len() > 0) {
        // result / extension = 1
        protected_data.push((u8_pchannel_result << 6) | 0x20);
        protected_data.extend_from_slice(&p_pchannel_data);
    } else {
        // result / extension = 0
        protected_data.push((u8_pchannel_result << 6));
    }

    let mut cipher = eax::Eax::<Aes128>::new(eax::aead::generic_array::GenericArray::from_slice(
        &p_psk_context.m_Tek.0,
    ));

    header[0] >>= 2;

    if let Ok(data) = cipher.encrypt(
        eax::aead::generic_array::GenericArray::from_slice(&au8_nonce),
        eax::aead::Payload {
            msg: &protected_data,
            aad: &header,
        },
    ) {
        header[0] <<= 2;
        let len = header.len() + 4 + data.len();
        header[2] = ((len >> 8) & 0x00FF) as u8;
        header[3] = (len & 0x00FF) as u8;
        p_protected_data.clear();
        p_protected_data.extend_from_slice(&header);
        p_protected_data.extend_from_slice(&au8_nonce[12..]);
        p_protected_data.extend_from_slice(&data);
        return true;
    }
    return false;
}

//   /**********************************************************************************************************************/
//   /** The EAP_PSK_Encode_Message1 primitive is used to encode the first EAP-PSK message (type 0)
//    * This message is sent from the server to device
//    ***********************************************************************************************************************
//    *
//    * @param u8Identifier Message identifier retrieved from the Request
//    *
//    * @param au8RandS RandS parameter built by the server
//    *
//    * @param au8IdS IdS parameter (the server identity)
//    *
//    * @param u16MemoryBufferLength size of the buffer which will be used for data encoding
//    *
//    * @param pMemoryBuffer OUT parameter; upon successful return contains the encoded message; this buffer should be previously
//    *                      allocated
//    *
//    * @return encoded length or 0 if encoding failed
//    *
//    **********************************************************************************************************************/
pub fn eap_psk_encode_message1(
    u8_identifier: u8,
    p_rand_s: &TEapPskRand,
    p_id_s: &TEapPskNetworkAccessIdentifierS,
    
) -> Vec<u8> {
    let mut v = vec![EAP_REQUEST, u8_identifier, 0, 0,EAP_PSK_IANA_TYPE, EAP_PSK_T0];
    v.extend_from_slice(&p_rand_s.0);
    v.extend_from_slice(&p_id_s.0);

    v[2] = ((v.len() >> 8) & 0x00FF) as u8;
    v[3] = (v.len() & 0x00FF) as u8;

    v
}

//   /**********************************************************************************************************************/
//   /** The EAP_PSK_Decode_Message2 primitive is used to decode the second EAP-PSK message (type 1) and also to check
//    * the MacP parameter
//    ***********************************************************************************************************************
//    *
//    **********************************************************************************************************************/
pub fn eap_psk_decode_message2(
    p_message: &[u8],
    p_psk_context: &TEapPskContext,
    p_id_s: &TEapPskNetworkAccessIdentifierS,
    p_rand_s: &mut TEapPskRand,
    p_rand_p: &mut TEapPskRand,
) -> bool {
    *p_rand_s = p_message[0..16].into();
    *p_rand_p = p_message[16..32].into();
    let au8_mac_p = &p_message[32..48];
    let id_p_uc_size = 8usize;

    let id_p = &p_message[48..(48 + id_p_uc_size)];

    let mut au8_seed = id_p.to_vec();
    au8_seed.extend_from_slice(&p_id_s.0);
    // compute MacP = CMAC-AES-128(AK, IdP||IdS||RandS||RandP)
    au8_seed.extend_from_slice(&p_rand_s.0);
    au8_seed.extend_from_slice(&p_rand_p.0);

    let mut mac = Cmac::<aes::Aes128>::new_from_slice(&p_psk_context.m_Ak.0).unwrap();
    mac.update(&au8_seed);
    let au8_expected_mac_p = mac.finalize();
    return  au8_expected_mac_p.into_bytes().to_vec() == au8_mac_p.to_vec();
}

//   /**********************************************************************************************************************/
//   /** The EAP_PSK_Encode_Message3 primitive is used to encode the third EAP-PSK message (type 2)
//    ***********************************************************************************************************************
//    *
//    **********************************************************************************************************************/
pub fn EAP_PSK_Encode_Message3(
    pPskContext: &TEapPskContext,
    u8Identifier: u8,
    pRandS: &TEapPskRand,
    pRandP: &TEapPskRand,
    pIds: &TEapPskNetworkAccessIdentifierS,
    u32Nonce: u32,
    u8PChannelResult: u8,
    pPChannelData: &[u8],    
) -> Option<Vec<u8>> {
    let mut mac = Cmac::<aes::Aes128>::new_from_slice(&pPskContext.m_Ak.0).unwrap();
    let mut au8Seed: Vec<u8> = Vec::new();
    au8Seed.append(pIds.0.to_vec().borrow_mut());
    au8Seed.append(pRandP.0.to_vec().borrow_mut());

    mac.update(&au8Seed);

    let au8MacS = mac.finalize();
    let auMacSLen = au8MacS.clone().into_bytes().len();

    let mut header: Vec<u8> = Vec::new();
    // encode the EAP header; length field will be set at the end of the block
    header.push(EAP_REQUEST);
    header.push(u8Identifier);
    header.push(0);
    header.push(0);
    header.push(EAP_PSK_IANA_TYPE);
    header.push(EAP_PSK_T2);
    header.append(pRandS.0.to_vec().borrow_mut());
    header.append(au8MacS.into_bytes().to_vec().borrow_mut());

    log::info!("pRandS : {:?}", pRandS.0);
    
    //   // prepare P-Channel content
    //   // nonce should be big endian
    // nonce should be big endian
    let mut au8Nonce = [0u8; 16];

    au8Nonce[12] = ((u32Nonce >> 24) & 0xFF) as u8;
    au8Nonce[13] = ((u32Nonce >> 16) & 0xFF) as u8;
    au8Nonce[14] = ((u32Nonce >> 8) & 0xFF) as u8;
    au8Nonce[15] = ((u32Nonce) & 0xFF) as u8;

    log::info!("au8Nonce : {:?}", au8Nonce);
    // protected data
    let mut protected_data: Vec<u8> = Vec::new();

    log::info!("pChannel Data : {:?}", pPChannelData);
    if (pPChannelData.len() > 0) {
        // result / extension = 1
        protected_data.push((u8PChannelResult << 6) | 0x20);
        protected_data.extend_from_slice(pPChannelData);        
    } else {
        // result / extension = 0
        protected_data.push((u8PChannelResult << 6));
    }

    let mut cipher = eax::Eax::<Aes128>::new(eax::aead::generic_array::GenericArray::from_slice(
        &pPskContext.m_Tek.0,
    ));

    log::info!("pPsdkContext.m_Tek.0 {:?}", pPskContext.m_Tek);
    let len = header.len() + 4 + protected_data.len() + 16 /*TAG */;
    header[2] = ((len >> 8) & 0x00FF) as u8;
    header[3] = (len & 0x00FF) as u8;

    header[0] >>= 2;

    if let Ok(data) = cipher.encrypt(
        eax::aead::generic_array::GenericArray::from_slice(&au8Nonce), 
        eax::aead::Payload {
            msg: &protected_data,
            aad: &header[0..(header.len()- auMacSLen)], //remove au8Mac
        }
    ) {
        // log::info!("data : {:X?}", data);
        let (payload, tag) = data.split_at(protected_data.len());
        header[0] <<= 2;
        let mut result_vec = vec![];
        result_vec.append(&mut header);
        result_vec.extend_from_slice(&au8Nonce[12..]);

        result_vec.extend_from_slice(tag);
        result_vec.extend_from_slice(payload);

        return Some(result_vec);
    }
    return None;
}

//   /**********************************************************************************************************************/
//   /**
//    ***********************************************************************************************************************
//    *
//    **********************************************************************************************************************/
pub fn EAP_PSK_Decode_Message4(
    pMessage: &Vec<u8>,
    pPskContext: &TEapPskContext,
    pHeader: &Vec<u8>,
    p_rand_s: &mut TEapPskRand,
    pu32Nonce: &mut u32,
    pu8PChannelResult: &mut u8,
    pPChannelData: &mut Vec<u8>,
) -> bool {
    let mut bRet = false;

    // TODO: review size (ARIB)
    if (pMessage.len() >= 41) {
        *p_rand_s = pMessage[0..16].to_vec().into();

        // decrypt P-CHANNEL
        // P-CHANNEL uses the TEK key
        let mut cipher = eax::Eax::<Aes128>::new(eax::aead::generic_array::GenericArray::from_slice(
            &pPskContext.m_Tek.0,
        ));
        let p_nonce = &pMessage[16..20]; 
        let tag = &pMessage[20..36];       
        let protected_data = &pMessage[36..];
        let mut au8_nonce = [0u8; 16];
        au8_nonce[12] = p_nonce[0];
        au8_nonce[13] = p_nonce[1];
        au8_nonce[14] = p_nonce[2];
        au8_nonce[15] = p_nonce[3];
        let mut header = pHeader[0..22].to_vec(); //TODO, is this fixed?
        header[0] >>= 2;

        log::info!("TEK : {:?}", pPskContext.m_Tek);
        log::info!("Nonce/IV : {:X?}", au8_nonce);
        log::info!("Header : {:X?}", header);
        log::info!("Data-enc : {:X?}", protected_data);
        log::info!("Tag : {:X?}", tag);
    
        let mut data_and_tag: Vec<u8> = Vec::new();
        data_and_tag.append(protected_data.to_vec().borrow_mut());
        data_and_tag.append(tag.to_vec().borrow_mut());

        if let Ok(data) = cipher.decrypt(
            eax::aead::generic_array::GenericArray::from_slice(&au8_nonce),
            eax::aead::Payload {
                msg: &data_and_tag,
                aad: &header,
            },
        ) {
            *pu8PChannelResult = (data[0] & 0xC0) >> 6;
            *pPChannelData = data[1..].to_vec();
            *pu32Nonce = u32::from_be_bytes([p_nonce[3], p_nonce[2], p_nonce[1], p_nonce[0]]);
            bRet = true;
        }
    }

    return bRet;
}

//   /**********************************************************************************************************************/
//   /** The EAP_PSK_Encode_EAP_Success primitive is used to encode the EAP success message
//    ***********************************************************************************************************************
//    *
//    **********************************************************************************************************************/
  pub fn EAP_PSK_Encode_EAP_Success(
    u8Identifier: u8,    
    ) -> Option<Vec<u8>>
  {
     Some(vec![EAP_SUCCESS, u8Identifier, 0, 4])
    // check the size of the buffer        
  }

//   /**********************************************************************************************************************/
//   /** The EAP_PSK_Encode_GMK_Activation primitive is used to encode the GMK activation message
//    ***********************************************************************************************************************
//    *
//    **********************************************************************************************************************/
  pub fn EAP_PSK_Encode_GMK_Activation(pPChannelData:&Vec<u8>, pMemoryBuffer: &mut Vec<u8>)
  {
    pMemoryBuffer.push(pPChannelData[0]);
    pMemoryBuffer.push(pPChannelData[1]);
    pMemoryBuffer.push(pPChannelData[2]);
  }


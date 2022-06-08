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
        let u: [u8; 16] = v.try_into().unwrap();

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

pub fn EAP_PSK_Initialize(pKey: &TEapPskKey, pPskContext: &mut TEapPskContext) -> bool {
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


pub fn EAP_PSK_InitializeTEK(pRandP: &TEapPskRand, pPskContext: &mut TEapPskContext) -> bool {
    log::trace!("->EAP_PSK_InitializeTEK : {:?}, {:?}", pRandP, pPskContext.m_Kdk.0);
    let encryptor = aes::Aes128::new_from_slice(&pPskContext.m_Kdk.0);
    if let Ok(encryptor) = encryptor {
        let mut v = aes::cipher::generic_array::GenericArray::from(pRandP.0);
        aes::cipher::BlockEncrypt::encrypt_block(&encryptor, &mut v);
        v[15] ^= 0x01;
        aes::cipher::BlockEncrypt::encrypt_block(&encryptor, &mut v);
        pPskContext.m_Tek.0 = v.into();
        log::trace!("->EAP_PSK_InitializeTEK : {:?}, {:?}", pRandP, pPskContext.m_Tek);
        true
    } else {
        log::warn!("EAP_PSK_InitializeTEK : Failed to get encryptor");
        false
    }
}

pub fn EAP_PSK_Decode_Message(
    pMessage: &Vec<u8>,
    pu8Code: &mut u8,
    pu8Identifier: &mut u8,
    pu8TSubfield: &mut u8,
    pEAPData: &mut Vec<u8>,
) -> bool {
    let mut bRet = false;

    if (pMessage.len() >= 4) {
        *pu8Code = pMessage[0];
        *pu8Identifier = pMessage[1];
        let u16EapMessageLength: usize = ((pMessage[2] as usize) << 8) | pMessage[3] as usize;

        // A message with the Length field set to a value larger than the number of received octets MUST be silently discarded.
        bRet = (u16EapMessageLength <= pMessage.len());

        if (bRet && (u16EapMessageLength >= 6)) {
            *pu8TSubfield = pMessage[5];
            *pEAPData = pMessage[6..].to_vec();
            bRet = (pMessage[4] == EAP_PSK_IANA_TYPE);
        }
    }

    return bRet;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn EAP_PSK_Decode_Message1(
    pMessage: &Vec<u8>,
    pRandS: &mut TEapPskRand,
    pIdS: &mut TEapPskNetworkAccessIdentifier,
) -> bool {
    let mut bRet = false;

    // check the length of the message
    if (pMessage.len() >= pRandS.0.len()) {
        *pRandS = pMessage[0..16].to_vec().into();
        //   memcpy(pRandS->m_au8Value, pMessage, sizeof(pRandS->m_au8Value));
        pIdS.0 = pMessage[16..].to_vec();
        bRet = true;
    }

    return bRet;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn EAP_PSK_Encode_Message2(
    pPskContext: &TEapPskContext,
    u8Identifier: u8,
    pRandS: &TEapPskRand,
    pRandP: &TEapPskRand,
    pIdS: &TEapPskNetworkAccessIdentifier,
    pIdP: &TEapPskNetworkAccessIdentifier,
    pMemoryBuffer: &mut Vec<u8>,
) {
    // check the size of the buffer
    if (pMemoryBuffer.capacity() >= 62) {
        //TODO panic or increase size?
        // compute first MacP = CMAC-AES-128(AK, IdP||IdS||RandS||RandP)
        let mut au8Seed: Vec<u8> = Vec::new();

        let mut mac = Cmac::<aes::Aes128>::new_from_slice(&pPskContext.m_Ak.0).unwrap();

        //   ret = cipher_wrapper_cipher_setup( &m_ctx, cipher_info );
        //   LOG_DBG(Log("\n cipher_wrapper_cipher_setup returned %d %d", ret, m_ctx.cipher_info->type));
        au8Seed.append(&mut pIdP.0.to_vec());
        au8Seed.append(&mut pIdS.0.to_vec());
        au8Seed.append(&mut pRandS.0.to_vec());
        au8Seed.append(&mut pRandP.0.to_vec());
        mac.update(&au8Seed);

        let au8MacP = mac.finalize();
        // encode the EAP header; length field will be set at the end of the block
        pMemoryBuffer.push(EAP_RESPONSE);
        pMemoryBuffer.push(u8Identifier);
        pMemoryBuffer.push(0);
        pMemoryBuffer.push(0);
        pMemoryBuffer.push(EAP_PSK_IANA_TYPE);
        pMemoryBuffer.push(EAP_PSK_T1);
        pMemoryBuffer.append(&mut pRandS.0.to_vec());
        pMemoryBuffer.append(&mut pRandP.0.to_vec());
        pMemoryBuffer.append(&mut au8MacP.into_bytes().to_vec());
        pMemoryBuffer.append(&mut pIdP.0.to_vec());

        //   // now update the EAP header length field
        pMemoryBuffer[2] = ((pMemoryBuffer.len() >> 8) & 0x00FF) as u8;
        pMemoryBuffer[3] = (pMemoryBuffer.len() & 0x00FF) as u8;

        //   UNUSED(ret);
    }
}

/**********************************************************************************************************************/
/** The EAP_PSK_Decode_Message3 primitive is used to decode the third EAP-PSK message (type 2)
 **********************************************************************************************************************/
pub fn EAP_PSK_Decode_Message3(
    pMessage: &Vec<u8>,
    pPskContext: &TEapPskContext,
    pHeader: Vec<u8>,
    pRandS: &mut TEapPskRand,
    pu32Nonce: &mut u32,
    pu8PChannelResult: &mut u8,
    pPChannelData: &mut Vec<u8>,
) -> bool {
    let mut bRet = false;

    if (pMessage.len() >= 59) {
        *pRandS = pMessage[0..16].to_vec().into();
        let macS = &pMessage[16..32];

        // verify MacS: MAC_S = CMAC-AES-128(AK, IdS||RandP)
        let mut au8Seed: Vec<u8> = Vec::new();

        let mut mac = Cmac::<aes::Aes128>::new_from_slice(&pPskContext.m_Ak.0).unwrap();

        au8Seed.append(&mut pPskContext.m_IdS.0.to_vec());
        au8Seed.append(&mut pPskContext.m_RandP.0.to_vec());
        mac.update(&au8Seed);
        let au8MacS = mac.finalize().into_bytes().to_vec();

        if au8MacS == macS {
            let key = eax::aead::generic_array::GenericArray::from_slice(&pPskContext.m_Tek.0);
            let pNonce = &pMessage[32..36];
            // let pTag = pMessage[36..52];
            let pProtectedData = &pMessage[36..];
            let mut au8Nonce: [u8; 16] = [0; 16];
            au8Nonce[12] = pNonce[0];
            au8Nonce[13] = pNonce[1];
            au8Nonce[14] = pNonce[2];
            au8Nonce[15] = pNonce[3];
            let mut cipher = eax::Eax::<Aes128>::new(key);
            // The protected data is the 22 bytes header of the EAP message.
            // The G3 specifies a slightly modified EAP header but in the same time
            // the specification requires to compute the authentication tag over the
            // on the original EAP header
            // So we change the header to make it "EAP compliant", we compute the
            // auth tag and then we change back the header

            // right shift Code field with 2 bits as indicated in the EAP specification
            let mut header = pHeader.clone();
            header[0] >>= 2; //TODO, check if header has any data

            if let Ok(data) = cipher.decrypt(
                GenericArray::from_slice(&au8Nonce),
                Payload {
                    msg: pProtectedData,
                    aad: &header,
                },
            ) {
                *pu8PChannelResult = (data[0] & 0xC0) >> 6;
                *pPChannelData = data[1..].to_vec();
                *pu32Nonce = u32::from_be_bytes([pNonce[3], pNonce[2], pNonce[1], pNonce[0]]);
            }
        }
    }
    return bRet;
}

/**********************************************************************************************************************/
/** The EAP_PSK_Encode_Message4 primitive is used to encode the second EAP-PSK message (type 3)
 **********************************************************************************************************************/
pub fn EAP_PSK_Encode_Message4(
    pPskContext: &TEapPskContext,
    u8Identifier: u8,
    pRandS: &TEapPskRand,
    u32Nonce: u32,
    u8PChannelResult: u8,
    pPChannelData: Vec<u8>,
    pProtectedData: &mut Vec<u8>,
) -> bool {
    let mut header: Vec<u8> = Vec::with_capacity(22);

    let mut au8Nonce = [0u8; 16];

    // encode the EAP header; length field will be set at the end of the block
    header.push(EAP_RESPONSE);
    header.push(u8Identifier);
    header.push(0);
    header.push(0);
    header.push(EAP_PSK_IANA_TYPE);
    header.push(EAP_PSK_T3);

    // EAP message: add PSK fields

    header.append(&mut pRandS.0.to_vec());

    // nonce should be big endian
    au8Nonce[12] = ((u32Nonce >> 24) & 0xFF) as u8;
    au8Nonce[13] = ((u32Nonce >> 16) & 0xFF) as u8;
    au8Nonce[14] = ((u32Nonce >> 8) & 0xFF) as u8;
    au8Nonce[15] = ((u32Nonce) & 0xFF) as u8;

    // protected data
    let mut protected_data: Vec<u8> = Vec::new();

    if (pPChannelData.len() > 0) {
        // result / extension = 1
        protected_data.push((u8PChannelResult << 6) | 0x20);
        protected_data.append(pPChannelData.clone().borrow_mut());
    } else {
        // result / extension = 0
        protected_data.push((u8PChannelResult << 6));
    }

    let mut cipher = eax::Eax::<Aes128>::new(eax::aead::generic_array::GenericArray::from_slice(
        &pPskContext.m_Tek.0,
    ));

    header[0] >>= 2;

    if let Ok(data) = cipher.encrypt(
        eax::aead::generic_array::GenericArray::from_slice(&au8Nonce),
        eax::aead::Payload {
            msg: &protected_data,
            aad: &header,
        },
    ) {
        header[0] <<= 2;
        let len = header.len() + 4 + data.len();
        header[2] = ((len >> 8) & 0x00FF) as u8;
        header[3] = (len & 0x00FF) as u8;
        pProtectedData.clear();
        pProtectedData.append(&mut header);
        pProtectedData.append(au8Nonce[12..].to_vec().borrow_mut());
        pProtectedData.append(data.clone().borrow_mut());
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
pub fn EAP_PSK_Encode_Message1(
    u8Identifier: u8,
    pRandS: &TEapPskRand,
    pIdS: &TEapPskNetworkAccessIdentifierS,
    out_message: &mut Vec<u8>,
) {
    out_message.clear();
    out_message.push(EAP_REQUEST);
    out_message.push(u8Identifier);
    out_message.push(0);
    out_message.push(0);
    out_message.push(EAP_PSK_IANA_TYPE);
    out_message.push(EAP_PSK_T0);
    out_message.append(pRandS.0.to_vec().borrow_mut());
    out_message.append(pIdS.0.to_vec().borrow_mut());

    out_message[2] = ((out_message.len() >> 8) & 0x00FF) as u8;
    out_message[3] = (out_message.len() & 0x00FF) as u8;
}

//   /**********************************************************************************************************************/
//   /** The EAP_PSK_Decode_Message2 primitive is used to decode the second EAP-PSK message (type 1) and also to check
//    * the MacP parameter
//    ***********************************************************************************************************************
//    *
//    **********************************************************************************************************************/
pub fn EAP_PSK_Decode_Message2(
    pMessage: &Vec<u8>,
    pPskContext: &TEapPskContext,
    pIdS: &TEapPskNetworkAccessIdentifierS,
    pRandS: &mut TEapPskRand,
    pRandP: &mut TEapPskRand,
) -> bool {
    *pRandS = pMessage[0..16].to_vec().into();
    *pRandP = pMessage[16..32].to_vec().into();
    let au8MacP = pMessage[32..48].to_vec();
    let idP_uc_size = 8usize;

    let idP = pMessage[48..(48 + idP_uc_size)].to_vec();

    let mut au8Seed = idP.to_vec();
    au8Seed.append(pIdS.0.to_vec().borrow_mut());
    // compute MacP = CMAC-AES-128(AK, IdP||IdS||RandS||RandP)
    au8Seed.append(pRandS.0.to_vec().borrow_mut());
    au8Seed.append(pRandP.0.to_vec().borrow_mut());

    let mut mac = Cmac::<aes::Aes128>::new_from_slice(&pPskContext.m_Ak.0).unwrap();
    mac.update(&au8Seed);
    let au8ExpectedMacP = mac.finalize();
    if au8ExpectedMacP.into_bytes().to_vec() == au8MacP.to_vec() {
        return true;
    }
    return false;
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
    pPChannelData: &Vec<u8>,
    pMemoryBuffer: &mut Vec<u8>,
) -> bool {
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

    log::trace!("pRandS : {:?}", pRandS.0);
    
    //   // prepare P-Channel content
    //   // nonce should be big endian
    // nonce should be big endian
    let mut au8Nonce = [0u8; 16];

    au8Nonce[12] = ((u32Nonce >> 24) & 0xFF) as u8;
    au8Nonce[13] = ((u32Nonce >> 16) & 0xFF) as u8;
    au8Nonce[14] = ((u32Nonce >> 8) & 0xFF) as u8;
    au8Nonce[15] = ((u32Nonce) & 0xFF) as u8;

    log::trace!("au8Nonce : {:?}", au8Nonce);
    // protected data
    let mut protected_data: Vec<u8> = Vec::new();

    log::trace!("pChannel Data : {:?}", pPChannelData);
    if (pPChannelData.len() > 0) {
        // result / extension = 1
        protected_data.push((u8PChannelResult << 6) | 0x20);
        protected_data.append(pPChannelData.clone().borrow_mut());
    } else {
        // result / extension = 0
        protected_data.push((u8PChannelResult << 6));
    }

    let mut cipher = eax::Eax::<Aes128>::new(eax::aead::generic_array::GenericArray::from_slice(
        &pPskContext.m_Tek.0,
    ));

    log::trace!("pPsdkContext.m_Tek.0 {:?}", pPskContext.m_Tek);
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
        // log::trace!("data : {:X?}", data);
        let (payload, tag) = data.split_at(protected_data.len());
        header[0] <<= 2;
        pMemoryBuffer.clear();
        pMemoryBuffer.append(&mut header);
        pMemoryBuffer.append(au8Nonce[12..].to_vec().borrow_mut());

        pMemoryBuffer.append(tag.to_vec().clone().borrow_mut());
        pMemoryBuffer.append(payload.to_vec().clone().borrow_mut());

        return true;
    }
    return false;
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
    pRandS: &mut TEapPskRand,
    pu32Nonce: &mut u32,
    pu8PChannelResult: &mut u8,
    pPChannelData: &mut Vec<u8>,
) -> bool {
    let mut bRet = false;

    // TODO: review size (ARIB)
    if (pMessage.len() >= 41) {
        *pRandS = pMessage[0..16].to_vec().into();

        // decrypt P-CHANNEL
        // P-CHANNEL uses the TEK key
        let mut cipher = eax::Eax::<Aes128>::new(eax::aead::generic_array::GenericArray::from_slice(
            &pPskContext.m_Tek.0,
        ));
        let pNonce = &pMessage[16..20]; 
        let tag = &pMessage[20..36];       
        let protected_data = &pMessage[36..];
        let mut au8Nonce = [0u8; 16];
        au8Nonce[12] = pNonce[0];
        au8Nonce[13] = pNonce[1];
        au8Nonce[14] = pNonce[2];
        au8Nonce[15] = pNonce[3];
        let mut header = pHeader[0..22].to_vec(); //TODO, is this fixed?
        header[0] >>= 2;

        log::trace!("TEK : {:?}", pPskContext.m_Tek);
        log::trace!("Nonce/IV : {:X?}", au8Nonce);
        log::trace!("Header : {:X?}", header);
        log::trace!("Data-enc : {:X?}", protected_data);
        log::trace!("Tag : {:X?}", tag);
    
        let mut data_and_tag: Vec<u8> = Vec::new();
        data_and_tag.append(protected_data.to_vec().borrow_mut());
        data_and_tag.append(tag.to_vec().borrow_mut());

        if let Ok(data) = cipher.decrypt(
            eax::aead::generic_array::GenericArray::from_slice(&au8Nonce),
            eax::aead::Payload {
                msg: &data_and_tag,
                aad: &header,
            },
        ) {
            *pu8PChannelResult = (data[0] & 0xC0) >> 6;
            *pPChannelData = data[1..].to_vec();
            *pu32Nonce = u32::from_be_bytes([pNonce[3], pNonce[2], pNonce[1], pNonce[0]]);
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
    pMemoryBuffer: &mut Vec<u8>
    )
  {
    pMemoryBuffer.clear();
    // check the size of the buffer
    pMemoryBuffer.push(EAP_SUCCESS);
    pMemoryBuffer.push(u8Identifier);
    pMemoryBuffer.push(0);
    pMemoryBuffer.push(4);
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


use crate::hislip::errors::{Error, FatalErrorCode, NonFatalErrorCode};
use byteorder::{BigEndian, ByteOrder, NetworkEndian};
use core::option::Option;
use core::option::Option::{None, Some};
use core::result::Result;
use core::result::Result::{Err, Ok};

#[derive(Debug, Copy, Clone)]
pub struct Message<'a> {
    pub(crate) message_type: MessageType,
    pub(crate) control_code: u8,
    pub(crate) message_parameter: u32,
    pub(crate) payload: &'a [u8],
}

impl<'a> Message<'a> {
    pub fn from_buffer(x: &mut [u8]) -> Result<(usize, Message), Error> {
        if x.len() < 16 {
            Err(Error::None)
        } else {
            let prolog = x.get(0..2).unwrap();
            if prolog != b"HS" {
                return Err(Error::Fatal(
                    FatalErrorCode::PoorlyFormattedMessageHeader,
                    b"Invalid prologue",
                ));
            }

            let len = BigEndian::read_u64(&x[8..16]) as usize;
            if x.len() < len + 16 {
                // Not finished receiving
                return Err(Error::None);
            }

            Ok((
                len + 16,
                Message {
                    message_type: MessageType::from_message_type(x[2]).ok_or(Error::NonFatal(
                        NonFatalErrorCode::UnrecognizedMessageType,
                        b"Bad message type",
                    ))?,
                    control_code: x[3],
                    message_parameter: BigEndian::read_u32(&mut x[4..8]),
                    payload: &x[16..len + 16],
                },
            ))
        }
    }

    pub fn fill_buffer(&self, x: &mut [u8]) -> (usize, ()) {
        let len = 16 + self.payload.len();
        if len <= x.len() {
            x[0] = b'H';
            x[1] = b'S';
            x[2] = self.message_type.get_message_type();
            x[3] = self.control_code;
            NetworkEndian::write_u32(&mut x[4..8], self.message_parameter);
            NetworkEndian::write_u64(&mut x[8..16], self.payload.len() as u64);
            x[16..len].copy_from_slice(self.payload);
        }
        (len, ())
    }
}

/// Message Type Value Definitions
///
/// See Table 4 in HiSLIP specification
#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    Initialize,
    InitializeResponse,
    FatalError,
    Error,
    AsyncLock,
    AsyncLockResponse,
    Data,
    DataEnd,
    DeviceClearComplete,
    DeviceClearAcknowledge,
    AsyncRemoteLocalControl,
    AsyncRemoteLocalResponse,
    Trigger,
    Interrupted,
    AsyncInterrupted,
    AsyncMaximumMessageSize,
    AsyncMaximumMessageSizeResponse,
    AsyncInitialize,
    AsyncInitializeResponse,
    AsyncDeviceClear,
    AsyncServiceRequest,
    AsyncStatusQuery,
    AsyncStatusResponse,
    AsyncDeviceClearAcknowledge,
    AsyncLockInfo,
    AsyncLockInfoResponse,
    GetDescriptors,
    GetDescriptorsResponse,
    StartTLS,
    AsyncStartTLS,
    AsyncStartTLSResponse,
    EndTLS,
    AsyncEndTLS,
    AsyncEndTLSResponse,
    GetSaslMechanismList,
    GetSaslMechanismListResponse,
    AuthenticationStart,
    AuthenticationExchange,
    AuthenticationResult,
    /// Vendor-specific, only codes 128-255 are allowed
    VendorSpecific(u8),
}

impl MessageType {
    pub fn get_message_type(&self) -> u8 {
        match self {
            MessageType::Initialize => 0,
            MessageType::InitializeResponse => 1,
            MessageType::FatalError => 2,
            MessageType::Error => 3,
            MessageType::AsyncLock => 4,
            MessageType::AsyncLockResponse => 5,
            MessageType::Data => 6,
            MessageType::DataEnd => 7,
            MessageType::DeviceClearComplete => 8,
            MessageType::DeviceClearAcknowledge => 9,
            MessageType::AsyncRemoteLocalControl => 10,
            MessageType::AsyncRemoteLocalResponse => 11,
            MessageType::Trigger => 12,
            MessageType::Interrupted => 13,
            MessageType::AsyncInterrupted => 14,
            MessageType::AsyncMaximumMessageSize => 15,
            MessageType::AsyncMaximumMessageSizeResponse => 16,
            MessageType::AsyncInitialize => 17,
            MessageType::AsyncInitializeResponse => 18,
            MessageType::AsyncDeviceClear => 19,
            MessageType::AsyncServiceRequest => 20,
            MessageType::AsyncStatusQuery => 21,
            MessageType::AsyncStatusResponse => 22,
            MessageType::AsyncDeviceClearAcknowledge => 23,
            MessageType::AsyncLockInfo => 24,
            MessageType::AsyncLockInfoResponse => 25,
            MessageType::GetDescriptors => 26,
            MessageType::GetDescriptorsResponse => 27,
            MessageType::StartTLS => 28,
            MessageType::AsyncStartTLS => 29,
            MessageType::AsyncStartTLSResponse => 30,
            MessageType::EndTLS => 31,
            MessageType::AsyncEndTLS => 32,
            MessageType::AsyncEndTLSResponse => 33,
            MessageType::GetSaslMechanismList => 34,
            MessageType::GetSaslMechanismListResponse => 35,
            MessageType::AuthenticationStart => 36,
            MessageType::AuthenticationExchange => 37,
            MessageType::AuthenticationResult => 38,
            MessageType::VendorSpecific(x) => x & 0x7F,
        }
    }

    pub fn from_message_type(typ: u8) -> Option<MessageType> {
        match typ {
            0 => Some(MessageType::Initialize),
            1 => Some(MessageType::InitializeResponse),
            2 => Some(MessageType::FatalError),
            3 => Some(MessageType::Error),
            4 => Some(MessageType::AsyncLock),
            5 => Some(MessageType::AsyncLockResponse),
            6 => Some(MessageType::Data),
            7 => Some(MessageType::DataEnd),
            8 => Some(MessageType::DeviceClearComplete),
            9 => Some(MessageType::DeviceClearAcknowledge),
            10 => Some(MessageType::AsyncRemoteLocalControl),
            11 => Some(MessageType::AsyncRemoteLocalResponse),
            12 => Some(MessageType::Trigger),
            13 => Some(MessageType::Interrupted),
            14 => Some(MessageType::AsyncInterrupted),
            15 => Some(MessageType::AsyncMaximumMessageSize),
            16 => Some(MessageType::AsyncMaximumMessageSizeResponse),
            17 => Some(MessageType::AsyncInitialize),
            18 => Some(MessageType::AsyncInitializeResponse),
            19 => Some(MessageType::AsyncDeviceClear),
            20 => Some(MessageType::AsyncServiceRequest),
            21 => Some(MessageType::AsyncStatusQuery),
            22 => Some(MessageType::AsyncStatusResponse),
            23 => Some(MessageType::AsyncDeviceClearAcknowledge),
            24 => Some(MessageType::AsyncLockInfo),
            25 => Some(MessageType::AsyncLockInfoResponse),
            26 => Some(MessageType::GetDescriptors),
            27 => Some(MessageType::GetDescriptorsResponse),
            28 => Some(MessageType::StartTLS),
            29 => Some(MessageType::AsyncStartTLS),
            30 => Some(MessageType::AsyncStartTLSResponse),
            31 => Some(MessageType::EndTLS),
            32 => Some(MessageType::AsyncEndTLS),
            33 => Some(MessageType::AsyncEndTLSResponse),
            34 => Some(MessageType::GetSaslMechanismList),
            35 => Some(MessageType::GetSaslMechanismListResponse),
            36 => Some(MessageType::AuthenticationStart),
            37 => Some(MessageType::AuthenticationExchange),
            38 => Some(MessageType::AuthenticationResult),
            128..=255 => Some(MessageType::VendorSpecific(typ)),
            _ => None,
        }
    }

    pub fn message(self, x: &[u8]) -> Message {
        Message {
            message_type: self,
            control_code: 0,
            message_parameter: 0,
            payload: x,
        }
    }

    pub fn message_params(self, x: &[u8], control_code: u8, message_parameter: u32) -> Message {
        Message {
            message_type: self,
            control_code,
            message_parameter,
            payload: x,
        }
    }
}

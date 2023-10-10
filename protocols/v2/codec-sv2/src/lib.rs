#![no_std]

extern crate alloc;

#[cfg(feature = "noise_sv2")]
use alloc::{boxed::Box, vec::Vec};

mod decoder;
mod encoder;
pub mod error;

pub use error::{CError, Error, Result};

pub use decoder::{StandardEitherFrame, StandardSv2Frame};

pub use decoder::StandardDecoder;
#[cfg(feature = "noise_sv2")]
pub use decoder::StandardNoiseDecoder;

pub use encoder::Encoder;
#[cfg(feature = "noise_sv2")]
pub use encoder::NoiseEncoder;

pub use framing_sv2::framing2::{Frame, Sv2Frame};
#[cfg(feature = "noise_sv2")]
pub use framing_sv2::framing2::{HandShakeFrame, NoiseFrame};

#[cfg(feature = "noise_sv2")]
pub use noise_sv2::{self, Initiator, NoiseCodec, Responder};

pub use buffer_sv2;

pub use framing_sv2;
use framing_sv2::framing2::handshake_message_to_frame as h2f;

#[cfg(feature = "noise_sv2")]
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum State {
    /// Not yet initialized
    NotInitialized,
    /// Handshake mode where codec is negotiating keys
    HandShake(HandshakeRole),
    /// Transport mode where AEAD is fully operational. The `TransportMode` object in this variant
    /// as able to perform encryption and decryption resp.
    Transport(NoiseCodec),
}
#[cfg(feature = "noise_sv2")]
impl State {
    pub fn step_0(&mut self) -> core::result::Result<HandShakeFrame, Error> {
        match self {
            Self::HandShake(h) => match h {
                HandshakeRole::Initiator(i) => i.step_0().map_err(|e| e.into()).map(h2f),
                HandshakeRole::Responder(_) => Err(Error::InvalidStepForResponder),
            },
            _ => Err(Error::NotInHandShakeState),
        }
    }

    pub fn step_1(&mut self, re_pub: [u8; 32]) -> core::result::Result<HandShakeFrame, Error> {
        match self {
            Self::HandShake(h) => match h {
                HandshakeRole::Responder(r) => r.step_1(re_pub).map_err(|e| e.into()).map(h2f),
                HandshakeRole::Initiator(_) => Err(Error::InvalidStepForInitiator),
            },
            _ => Err(Error::NotInHandShakeState),
        }
    }

    pub fn step_2(&mut self, message: [u8; 170]) -> core::result::Result<HandShakeFrame, Error> {
        match self {
            Self::HandShake(h) => match h {
                HandshakeRole::Initiator(i) => i.step_2(message).map_err(|e| e.into()).map(h2f),
                HandshakeRole::Responder(_) => Err(Error::InvalidStepForResponder),
            },
            _ => Err(Error::NotInHandShakeState),
        }
    }

    pub fn step_3(
        self,
        cipher_list: Vec<u8>,
    ) -> core::result::Result<(HandShakeFrame, Self), crate::error::Error> {
        match self {
            Self::HandShake(h) => match h {
                HandshakeRole::Responder(r) => {
                    let (message, codec) = r.step_3(cipher_list)?;
                    Ok((h2f(message), Self::Transport(codec)))
                }
                HandshakeRole::Initiator(_) => Err(Error::InvalidStepForInitiator),
            },
            _ => Err(Error::NotInHandShakeState),
        }
    }

    pub fn step_4(self, cipher_chosed: Vec<u8>) -> core::result::Result<Self, Error> {
        match self {
            Self::HandShake(h) => match h {
                HandshakeRole::Initiator(r) => {
                    let codec = r.step_4(cipher_chosed)?;
                    Ok(Self::Transport(codec))
                }
                HandshakeRole::Responder(_) => Err(Error::InvalidStepForResponder),
            },
            _ => Err(Error::NotInHandShakeState),
        }
    }
}
#[allow(clippy::large_enum_variant)]
#[cfg(feature = "noise_sv2")]
#[derive(Debug)]
pub enum HandshakeRole {
    Initiator(Box<noise_sv2::Initiator>),
    Responder(Box<noise_sv2::Responder>),
}

#[cfg(feature = "noise_sv2")]
impl State {
    pub fn take(&mut self) -> Self {
        core::mem::replace(self, Self::NotInitialized)
    }

    pub fn new() -> Self {
        Self::NotInitialized
    }

    pub fn initialize(inner: HandshakeRole) -> Self {
        Self::HandShake(inner)
    }

    pub fn with_transport_mode(tm: NoiseCodec) -> Self {
        Self::Transport(tm)
    }
}

#[cfg(feature = "noise_sv2")]
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(feature = "noise_sv2")]
mod tests {
    use super::*;

    #[test]
    fn handshake_step_fails_if_state_is_not_initialized() {
        let mut state = State::new();
        let actual = state.step_0().unwrap_err();
        let expect = Error::NotInHandShakeState;
        assert_eq!(actual, expect);
    }

    #[test]
    fn handshake_step_fails_if_state_is_in_transport_mode() {
        let mut state = State::new();
        let actual = state.step_0().unwrap_err();
        let expect = Error::NotInHandShakeState;
        assert_eq!(actual, expect);
    }

    #[test]
    fn into_transport_mode_errs_if_state_is_not_initialized() {
        let state = State::new();
        let actual = state.step_4(alloc::vec::Vec::new()).unwrap_err();
        let expect = Error::NotInHandShakeState;
        assert_eq!(actual, expect);
    }
}

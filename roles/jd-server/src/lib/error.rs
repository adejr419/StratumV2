use std::{
    convert::From,
    fmt::Debug,
    sync::{MutexGuard, PoisonError},
};

use roles_logic_sv2::parsers::Mining;

use crate::mempool::error::JdsMempoolError;

pub type JdsResult<T> = core::result::Result<T, JdsError>;

#[derive(std::fmt::Debug)]
pub enum JdsError {
    ConfigError(config::ConfigError),
    Io(std::io::Error),
    ChannelSend(Box<dyn std::marker::Send + Debug>),
    ChannelRecv(async_channel::RecvError),
    BinarySv2(binary_sv2::Error),
    Codec(codec_sv2::Error),
    Noise(noise_sv2::Error),
    RolesLogic(roles_logic_sv2::Error),
    Framing(codec_sv2::framing_sv2::Error),
    PoisonLock(String),
    Custom(String),
    Sv2ProtocolError((u32, Mining<'static>)),
    MempoolError(JdsMempoolError),
    ImpossibleToReconstructBlock(String),
    NoLastDeclaredJob,
}

impl std::fmt::Display for JdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use JdsError::*;
        match self {
            ConfigError(e) => write!(f, "Config error: {:?}", e),
            Io(ref e) => write!(f, "I/O error: `{:?}", e),
            ChannelSend(ref e) => write!(f, "Channel send failed: `{:?}`", e),
            ChannelRecv(ref e) => write!(f, "Channel recv failed: `{:?}`", e),
            BinarySv2(ref e) => write!(f, "Binary SV2 error: `{:?}`", e),
            Codec(ref e) => write!(f, "Codec SV2 error: `{:?}", e),
            Framing(ref e) => write!(f, "Framing SV2 error: `{:?}`", e),
            Noise(ref e) => write!(f, "Noise SV2 error: `{:?}", e),
            RolesLogic(ref e) => write!(f, "Roles Logic SV2 error: `{:?}`", e),
            PoisonLock(ref e) => write!(f, "Poison lock: {:?}", e),
            Custom(ref e) => write!(f, "Custom SV2 error: `{:?}`", e),
            Sv2ProtocolError(ref e) => {
                write!(f, "Received Sv2 Protocol Error from upstream: `{:?}`", e)
            }
            MempoolError(ref e) => write!(f, "Mempool error: `{:?}`", e),
            ImpossibleToReconstructBlock(e) => {
                write!(f, "Error in reconstructing the block: {:?}", e)
            }
            NoLastDeclaredJob => write!(f, "Last declared job not found"),
        }
    }
}

impl From<config::ConfigError> for JdsError {
    fn from(e: config::ConfigError) -> JdsError {
        JdsError::ConfigError(e)
    }
}

impl From<std::io::Error> for JdsError {
    fn from(e: std::io::Error) -> JdsError {
        JdsError::Io(e)
    }
}

impl From<async_channel::RecvError> for JdsError {
    fn from(e: async_channel::RecvError) -> JdsError {
        JdsError::ChannelRecv(e)
    }
}

impl From<binary_sv2::Error> for JdsError {
    fn from(e: binary_sv2::Error) -> JdsError {
        JdsError::BinarySv2(e)
    }
}

impl From<codec_sv2::Error> for JdsError {
    fn from(e: codec_sv2::Error) -> JdsError {
        JdsError::Codec(e)
    }
}

impl From<noise_sv2::Error> for JdsError {
    fn from(e: noise_sv2::Error) -> JdsError {
        JdsError::Noise(e)
    }
}

impl From<roles_logic_sv2::Error> for JdsError {
    fn from(e: roles_logic_sv2::Error) -> JdsError {
        JdsError::RolesLogic(e)
    }
}

impl<T: 'static + std::marker::Send + Debug> From<async_channel::SendError<T>> for JdsError {
    fn from(e: async_channel::SendError<T>) -> JdsError {
        JdsError::ChannelSend(Box::new(e))
    }
}

impl From<String> for JdsError {
    fn from(e: String) -> JdsError {
        JdsError::Custom(e)
    }
}
impl From<codec_sv2::framing_sv2::Error> for JdsError {
    fn from(e: codec_sv2::framing_sv2::Error) -> JdsError {
        JdsError::Framing(e)
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for JdsError {
    fn from(e: PoisonError<MutexGuard<T>>) -> JdsError {
        JdsError::PoisonLock(e.to_string())
    }
}

impl From<(u32, Mining<'static>)> for JdsError {
    fn from(e: (u32, Mining<'static>)) -> Self {
        JdsError::Sv2ProtocolError(e)
    }
}

impl From<JdsMempoolError> for JdsError {
    fn from(error: JdsMempoolError) -> Self {
        JdsError::MempoolError(error)
    }
}

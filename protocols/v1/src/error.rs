use crate::methods::{Method, MethodError};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Method(MethodError),
    InvalidJsonRpcMessageKind,
    InvalidSubmission,
    InvalidState(String),
    #[allow(clippy::upper_case_acronyms)]
    UnknownID(String),
    InvalidReceiver(Method),
}

impl From<MethodError> for Error {
    fn from(inner: MethodError) -> Self {
        Error::Method(inner)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!()
    }
}

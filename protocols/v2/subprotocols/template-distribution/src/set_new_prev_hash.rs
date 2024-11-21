#[cfg(not(feature = "with_serde"))]
use alloc::vec::Vec;
#[cfg(not(feature = "with_serde"))]
use binary_sv2::binary_codec_sv2::{self, free_vec, CVec};
#[cfg(not(feature = "with_serde"))]
use binary_sv2::Error;
use binary_sv2::{Deserialize, Serialize, U256};
#[cfg(not(feature = "with_serde"))]
use core::convert::TryInto;

/// Message used by an upstream to indicate the latest block header hash to mine on.
///
/// Upon validating a new best block, the server **must** immediately send this message.
///
/// If a `NewMiningJob` or `NewExtendedMiningJob` message has previously been sent with the
/// `future_job` flag set, the [`SetNewPrevHash::template_id`] field **should** be set to the
/// `template_id` present in that message.
///
/// Upon receiving this message, the downstream **must** begin mining on the template referenced If
/// they already received a `NewMiningJob` or `NewExtendedMiningJob` message with the `future_job`
/// flag set.
///
/// TODO: Define how many previous works the client has to track (2? 3?), and require that the
/// server reference one of those in SetNewPrevHash.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetNewPrevHash<'decoder> {
    /// Identifier of the template to mine on.
    ///
    /// This must be identical to previously sent [`crate::NewTemplate`] message.
    pub template_id: u64,
    /// Previous block’s hash, as it must appear in the next block’s header.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub prev_hash: U256<'decoder>,
    /// nTime field in the block header at which the client should start (usually current time).
    ///
    /// This is **not** the minimum valid nTime value.
    pub header_timestamp: u32,
    /// Block header field.
    pub n_bits: u32,
    /// The maximum double-SHA256 hash value which would represent a valid block. Note that this
    /// may be lower than the target implied by nBits in several cases, including weak-block based
    /// block propagation.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub target: U256<'decoder>,
}

/// Message used by an upstream to indicate the latest block header hash to mine on.
///
/// Upon validating a new best block, the server **must** immediately send this message.
///
/// If a `NewMiningJob` or `NewExtendedMiningJob` message has previously been sent with the
/// Future Job flag set, the [`SetNewPrevHash::template_id`] field **should** be set to the
/// `template_id` present in that message.
///
/// Upon receiving this message, the downstream **must** begin mining on the template referenced If
/// they already received a `NewMiningJob` or `NewExtendedMiningJob` message with the Future Job
/// flag set.
///
/// TODO: Define how many previous works the client has to track (2? 3?), and require that the
/// server reference one of those in SetNewPrevHash.
#[cfg(not(feature = "with_serde"))]
#[repr(C)]
pub struct CSetNewPrevHash {
    template_id: u64,
    prev_hash: CVec,
    header_timestamp: u32,
    n_bits: u32,
    target: CVec,
}

#[cfg(not(feature = "with_serde"))]
impl<'a> CSetNewPrevHash {
    /// Converts CSetNewPrevHash(C representation) to SetNewPrevHash(Rust representation).
    #[cfg(not(feature = "with_serde"))]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_rust_rep_mut(&'a mut self) -> Result<SetNewPrevHash<'a>, Error> {
        let prev_hash: U256 = self.prev_hash.as_mut_slice().try_into()?;
        let target: U256 = self.target.as_mut_slice().try_into()?;

        Ok(SetNewPrevHash {
            template_id: self.template_id,
            prev_hash,
            header_timestamp: self.header_timestamp,
            n_bits: self.n_bits,
            target,
        })
    }
}

/// Drops the CSetNewPrevHash object.
#[no_mangle]
#[cfg(not(feature = "with_serde"))]
pub extern "C" fn free_set_new_prev_hash(s: CSetNewPrevHash) {
    drop(s)
}

#[cfg(not(feature = "with_serde"))]
impl Drop for CSetNewPrevHash {
    fn drop(&mut self) {
        free_vec(&mut self.target);
    }
}

#[cfg(not(feature = "with_serde"))]
impl<'a> From<SetNewPrevHash<'a>> for CSetNewPrevHash {
    fn from(v: SetNewPrevHash<'a>) -> Self {
        Self {
            template_id: v.template_id,
            prev_hash: v.prev_hash.into(),
            header_timestamp: v.header_timestamp,
            n_bits: v.n_bits,
            target: v.target.into(),
        }
    }
}
#[cfg(feature = "with_serde")]
use binary_sv2::GetSize;
#[cfg(feature = "with_serde")]
impl<'d> GetSize for SetNewPrevHash<'d> {
    fn get_size(&self) -> usize {
        self.template_id.get_size()
            + self.prev_hash.get_size()
            + self.header_timestamp.get_size()
            + self.n_bits.get_size()
            + self.target.get_size()
    }
}

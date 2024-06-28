#[cfg(not(feature = "with_serde"))]
use alloc::vec::Vec;
#[cfg(not(feature = "with_serde"))]
use binary_sv2::binary_codec_sv2;
use binary_sv2::{Deserialize, Serialize, Str0255, B0255, B064K};
#[cfg(not(feature = "with_serde"))]
use core::convert::TryInto;

#[cfg(doc)]
use crate::{DeclareMiningJob, DeclareMiningJobSuccess};
#[cfg(doc)]
use common_messages_sv2::SetupConnection;
#[cfg(doc)]
use mining_sv2::SetCustomMiningJob;
#[cfg(doc)]
use template_distribution_sv2::CoinbaseOutputDataSize;

/// A request to get an identifier for a future-submitted mining job.
/// Rate limited to a rather slow rate and only available on connections where this has been
/// negotiated. Otherwise, only `mining_job_token(s)` from [`AllocateMiningJobTokenSuccess`] are valid.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct AllocateMiningJobToken<'decoder> {
    /// Unconstrained sequence of bytes. Whatever is needed by the pool to identify/authenticate
    /// the client, e.g. "braiinstest". Additional restrictions can be imposed by the pool. It is
    /// highly recommended that UTF-8 encoding is used.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub user_identifier: Str0255<'decoder>,
    /// Unique identifier for pairing the response
    pub request_id: u32,
}

/// The Server MUST NOT change the value of `coinbase_output_max_additional_size` in
/// [`AllocateMiningJobTokenSuccess`] messages unless required for changes to the pool’
/// configuration.
/// Notably, if the pool intends to change the space it requires for coinbase transaction outputs
/// regularly, it should simply prefer to use the maximum of all such output sizes as the
/// `coinbase_output_max_additional_size` value.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct AllocateMiningJobTokenSuccess<'decoder> {
    /// Unique identifier for pairing the response
    pub request_id: u32,
    /// Token that makes the client eligible for committing a mining job for approval/transaction
    /// declaration or for identifying custom mining job on mining connection.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub mining_job_token: B0255<'decoder>,
    /// The maximum additional serialized bytes which the pool will add in coinbase transaction
    /// outputs. See discussion in the Template Distribution Protocol's
    /// [`CoinbaseOutputDataSize`] message for more details.
    pub coinbase_output_max_additional_size: u32,
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    /// Bitcoin transaction outputs added by the pool
    pub coinbase_output: B064K<'decoder>,
    /// If true, the [`mining_job_token`](#structfield.mining_job_token) can be used immediately on a mining connection in the
    /// [`SetCustomMiningJob`] message, even before [`DeclareMiningJob`] and [`DeclareMiningJobSuccess`]
    /// messages have been sent and received. If false, Job Declarator MUST use this token for
    /// [`DeclareMiningJob`] only.
    /// This MUST be true when [`SetupConnection`] flags had REQUIRES_ASYNC_JOB_MINING set.
    pub async_mining_allowed: bool,
}

#[cfg(feature = "with_serde")]
use binary_sv2::GetSize;
#[cfg(feature = "with_serde")]
impl<'d> GetSize for AllocateMiningJobToken<'d> {
    fn get_size(&self) -> usize {
        self.user_identifier.get_size() + self.request_id.get_size()
    }
}
#[cfg(feature = "with_serde")]
impl<'d> GetSize for AllocateMiningJobTokenSuccess<'d> {
    fn get_size(&self) -> usize {
        self.request_id.get_size()
            + self.mining_job_token.get_size()
            + self.coinbase_output_max_additional_size.get_size()
            + self.coinbase_output.get_size()
            + self.async_mining_allowed.get_size()
    }
}

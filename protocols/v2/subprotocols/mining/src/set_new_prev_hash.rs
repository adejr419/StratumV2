#[cfg(not(feature = "with_serde"))]
use alloc::vec::Vec;
#[cfg(not(feature = "with_serde"))]
use binary_sv2::binary_codec_sv2;
use binary_sv2::{Deserialize, Serialize, U256};
#[cfg(not(feature = "with_serde"))]
use core::convert::TryInto;

/// Used by upstream to share the previous block hash whenever a new block is detected in the
/// network.
///
/// This message may be shared by all downstream nodes (sent only once to each channel group).
///
/// Downstream must immediately start to mine on the provided [`SetNewPrevHash::prevhash`].
///
/// When a downstream receives this message, only the job referenced by [`SetNewPrevHash::job_id`]
/// is valid. The remaining jobs already queued by the downstream have to be dropped.
///
/// Note: There is no need for block height in this message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetNewPrevHash<'decoder> {
    /// Group channel or channel that this prevhash is valid for.
    pub channel_id: u32,
    /// Job identfier that is to be used for mining with this prevhash.
    ///
    /// A pool may have provided multiple jobs for the next block height (e.g. an empty block or a
    /// block with transactions that are complementary to the set of transactions present in the
    /// current block template).
    pub job_id: u32,
    /// Previous block’s hash, block header field.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub prev_hash: U256<'decoder>,
    /// Smallest nTime value available for hashing.
    pub min_ntime: u32,
    /// Block header field.
    pub nbits: u32,
}

#[cfg(feature = "with_serde")]
use binary_sv2::GetSize;
#[cfg(feature = "with_serde")]
impl<'d> GetSize for SetNewPrevHash<'d> {
    fn get_size(&self) -> usize {
        self.channel_id.get_size()
            + self.job_id.get_size()
            + self.prev_hash.get_size()
            + self.min_ntime.get_size()
            + self.nbits.get_size()
    }
}
#[cfg(feature = "with_serde")]
impl<'a> SetNewPrevHash<'a> {
    pub fn into_static(self) -> SetNewPrevHash<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
    pub fn as_static(&self) -> SetNewPrevHash<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
}

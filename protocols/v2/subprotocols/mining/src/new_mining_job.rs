#[cfg(not(feature = "with_serde"))]
use alloc::vec::Vec;
#[cfg(not(feature = "with_serde"))]
use binary_sv2::binary_codec_sv2;
use binary_sv2::{Deserialize, Seq0255, Serialize, Sv2Option, B032, B064K, U256};
#[cfg(not(feature = "with_serde"))]
use core::convert::TryInto;

/// Used by an upstream to provide an updated mining job to the downstream through a standard
/// channel.
///
/// If the [`NewMiningJob::min_ntime`] is unset, then this indicates this is a future job.
/// If the [`NewMiningJob::min_ntime`] field is set, the downstream must immediately start mining the
/// new job after receiving this message, and use the value for the initial `nTime`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewMiningJob<'decoder> {
    /// Channel identifier for the channel that this job is valid for.
    ///
    /// This must be a standard channel.
    pub channel_id: u32,
    /// Upstream’s identification of the mining job.
    ///
    /// This identifier must be provided to the upstream when shares are submitted later in the
    /// mining process.
    pub job_id: u32,
    /// Smallest `nTime` value available for hashing for the new mining job.
    ///
    /// An empty value indicates this is a future job to be activated once a [`SetNewPrevHash`] message
    /// is received with a matching `job_id`. This [`SetNewPrevHash`] message provides the new `prev_hash`
    /// and `min_ntime`.
    ///
    /// Otherwise, if [`NewMiningJob::min_ntime`] value is set, this mining job is active and miner
    /// must start mining on it immediately. In this case, the new mining job uses the `prev_hash`
    /// from the last received [`SetNewPrevHash`] message.
    ///
    /// [`SetNewPrevHash`]: crate::SetNewPrevHash
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub min_ntime: Sv2Option<'decoder, u32>,
    /// Version field that reflects the current network consensus.
    ///
    /// As specified in [BIP320](https://github.com/bitcoin/bips/blob/master/bip-0320.mediawiki),
    /// the general purpose bits can be freely manipulated by the downstream node.
    ///
    /// The downstream node must not rely on the upstream node to set the
    /// [BIP320](https://github.com/bitcoin/bips/blob/master/bip-0320.mediawiki) bits to any
    /// particular value.
    pub version: u32,
    /// Merkle root field as used in the bitcoin block header.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub merkle_root: B032<'decoder>,
}

impl<'d> NewMiningJob<'d> {
    pub fn is_future(&self) -> bool {
        self.min_ntime.clone().into_inner().is_none()
    }
    pub fn set_future(&mut self) {
        self.min_ntime = Sv2Option::new(None);
    }
    pub fn set_no_future(&mut self, min_ntime: u32) {
        self.min_ntime = Sv2Option::new(Some(min_ntime));
    }
}

/// Used by an upstream to provide an updated mining job to the downstream through an
/// extended or group channel only.
///
/// Extended Channel: The whole search space of the job is owned by the specified channel. If the
/// future job field is set to false, i.e., [`NewExtendedMiningJob::min_ntime`] is not empty, the
/// downstream must start to mine on the new job as soon as they receive this message.
///
/// In the Extended Channel scenario, the full coinbase transaction can be constructed as of the
/// following: `extranonce_prefix + extranonce`(=N bytes), where N is the negotiated extranonce
/// space for the channel [`extranonce_size`].
///
/// Group Channel: This is a broadcast variant of [`NewMiningJob`] message with the `merkle_root`
/// field replaced by [`NewExtendedMiningJob::merkle_path`], [`NewExtendedMiningJob::
/// coinbase_tx_prefix`] and [`NewExtendedMiningJob::coinbase_tx_suffix`] for further traffic
/// optimization.
///
/// The Merkle root is then defined deterministically for each channel by the common
/// [`NewExtendedMiningJob::merkle_path`] and unique `extranonce_prefix` serialized into the
/// coinbase.
///
/// The full coinbase can then be constructed as the following:
/// [`NewExtendedMiningJob::coinbase_tx_prefix`] + `extranonce_prefix` +
/// [`NewExtendedMiningJob::coinbase_tx_suffix`].
///
/// A proxy may transform this multicast variant for downstream standard channels into
/// [`NewMiningJob`] messages by computing the derived Merkle root for them. A proxy must translate
/// the message for all downstream channels belonging to the group which don’t signal that they
/// accept extended mining jobs in the `SetupConnection` message (intended and expected behaviour
/// for end mining devices).
///
/// [`extranonce_size`]: crate::OpenExtendedMiningChannelSuccess::extranonce_size
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewExtendedMiningJob<'decoder> {
    /// Identifier of the Extended Mining Channel that this job is valid for.
    ///
    /// For a Group Channel, the message is broadcasted to all standard channels belonging to the
    /// group.
    pub channel_id: u32,
    /// Upstream’s identification of the mining job.
    ///
    /// This identifier must be provided to the upstream when shares are submitted later in the
    /// mining process.
    pub job_id: u32,
    /// Smallest nTime value available for hashing for the new mining job.
    ///
    /// An empty value indicates this is a future job to be activated once a [`SetNewPrevHash`] message
    /// is received with a matching `job_id`. This [`SetNewPrevHash`] message provides the new `prev_hash`
    /// and `min_ntime`.
    ///
    /// Otherwise, if [`NewMiningJob::min_ntime`] value is set, this mining job is active and miner
    /// must start mining on it immediately. In this case, the new mining job uses the `prev_hash`
    /// from the last received [`SetNewPrevHash`] message.
    ///
    /// [`SetNewPrevHash`]: crate::messages::SetNewPrevHash
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub min_ntime: Sv2Option<'decoder, u32>,
    /// Version field that reflects the current network consensus.
    ///
    /// As specified in [BIP320](https://github.com/bitcoin/bips/blob/master/bip-0320.mediawiki),
    /// the general purpose bits can be freely manipulated by the downstream node.
    ///
    /// The downstream node must not rely on the upstream node to set the
    /// [BIP320](https://github.com/bitcoin/bips/blob/master/bip-0320.mediawiki) bits to any
    /// particular value.
    pub version: u32,
    /// If set to `true`, the general purpose bits of [`NewExtendedMiningJob::version`] (as
    /// specified in BIP320) can be freely manipulated by the downstream node.
    ///
    /// If set to `false`, the downstream node must use [`NewExtendedMiningJob::version`] as it is
    /// defined by this message.
    pub version_rolling_allowed: bool,
    /// Merkle path hashes ordered from deepest.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub merkle_path: Seq0255<'decoder, U256<'decoder>>,
    /// Prefix part of the coinbase transaction.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub coinbase_tx_prefix: B064K<'decoder>,
    /// Suffix part of the coinbase transaction.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub coinbase_tx_suffix: B064K<'decoder>,
}

impl<'d> NewExtendedMiningJob<'d> {
    pub fn is_future(&self) -> bool {
        self.min_ntime.clone().into_inner().is_none()
    }
    pub fn set_future(&mut self) {
        self.min_ntime = Sv2Option::new(None);
    }
    pub fn set_no_future(&mut self, min_ntime: u32) {
        self.min_ntime = Sv2Option::new(Some(min_ntime));
    }
}

#[cfg(feature = "with_serde")]
use binary_sv2::GetSize;
#[cfg(feature = "with_serde")]
impl<'d> GetSize for NewExtendedMiningJob<'d> {
    fn get_size(&self) -> usize {
        self.channel_id.get_size()
            + self.job_id.get_size()
            + self.min_ntime.get_size()
            + self.version.get_size()
            + self.version_rolling_allowed.get_size()
            + self.merkle_path.get_size()
            + self.coinbase_tx_prefix.get_size()
            + self.coinbase_tx_suffix.get_size()
    }
}
#[cfg(feature = "with_serde")]
impl<'d> GetSize for NewMiningJob<'d> {
    fn get_size(&self) -> usize {
        self.channel_id.get_size()
            + self.job_id.get_size()
            + self.min_ntime.get_size()
            + self.version.get_size()
            + self.merkle_root.get_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::from_arbitrary_vec_to_array;
    use core::convert::TryFrom;
    use quickcheck_macros;

    #[quickcheck_macros::quickcheck]
    fn test_new_extended_mining_job(
        channel_id: u32,
        job_id: u32,
        min_ntime: Option<u32>,
        version: u32,
        version_rolling_allowed: bool,
        merkle_path: Vec<u8>,
        mut coinbase_tx_prefix: Vec<u8>,
        mut coinbase_tx_suffix: Vec<u8>,
    ) -> bool {
        let merkle_path = helpers::scan_to_u256_sequence(&merkle_path);
        let coinbase_tx_prefix = helpers::bytes_to_b064k(&mut coinbase_tx_prefix);
        let coinbase_tx_suffix = helpers::bytes_to_b064k(&mut coinbase_tx_suffix);
        let nemj = NewExtendedMiningJob {
            channel_id,
            job_id,
            min_ntime: Sv2Option::new(min_ntime),
            version,
            version_rolling_allowed,
            merkle_path: merkle_path.clone(),
            coinbase_tx_prefix: coinbase_tx_prefix.clone(),
            coinbase_tx_suffix: coinbase_tx_suffix.clone(),
        };
        let static_nmj = nemj.as_static();
        static_nmj.channel_id == nemj.channel_id
            && static_nmj.job_id == nemj.job_id
            && static_nmj.min_ntime == nemj.min_ntime
            && static_nmj.version == nemj.version
            && static_nmj.version_rolling_allowed == nemj.version_rolling_allowed
            && static_nmj.merkle_path == merkle_path
            && static_nmj.coinbase_tx_prefix == coinbase_tx_prefix
            && static_nmj.coinbase_tx_suffix == coinbase_tx_suffix
    }

    #[quickcheck_macros::quickcheck]
    fn test_new_mining_job(
        channel_id: u32,
        job_id: u32,
        min_ntime: Option<u32>,
        version: u32,
        merkle_root: Vec<u8>,
    ) -> bool {
        let merkle_root = from_arbitrary_vec_to_array(merkle_root);
        let nmj = NewMiningJob {
            channel_id,
            job_id,
            min_ntime: Sv2Option::new(min_ntime),
            version,
            merkle_root: B032::try_from(merkle_root.to_vec())
                .expect("NewMiningJob: failed to convert merkle_root to B032"),
        };
        let static_nmj = nmj.clone().as_static();
        static_nmj.channel_id == nmj.channel_id
            && static_nmj.job_id == nmj.job_id
            && static_nmj.min_ntime == nmj.min_ntime
            && static_nmj.version == nmj.version
            && static_nmj.merkle_root == nmj.merkle_root
    }

    pub mod helpers {
        use super::*;

        pub fn scan_to_u256_sequence(bytes: &Vec<u8>) -> Seq0255<U256> {
            let inner: Vec<U256> = bytes
                .chunks(32)
                .map(|chunk| {
                    let data = from_arbitrary_vec_to_array(chunk.to_vec());
                    return U256::from(data);
                })
                .collect();
            Seq0255::new(inner).expect("Could not convert bytes to SEQ0255<U256")
        }

        pub fn bytes_to_b064k(bytes: &Vec<u8>) -> B064K {
            B064K::try_from(bytes.clone()).expect("Failed to convert to B064K")
        }
    }
}
#[cfg(feature = "with_serde")]
impl<'a> NewExtendedMiningJob<'a> {
    pub fn into_static(self) -> NewExtendedMiningJob<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
    pub fn as_static(&self) -> NewExtendedMiningJob<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
}
#[cfg(feature = "with_serde")]
impl<'a> NewMiningJob<'a> {
    pub fn into_static(self) -> NewMiningJob<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
    pub fn as_static(&self) -> NewMiningJob<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
}

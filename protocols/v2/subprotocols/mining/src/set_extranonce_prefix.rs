#[cfg(not(feature = "with_serde"))]
use alloc::vec::Vec;
#[cfg(not(feature = "with_serde"))]
use binary_sv2::binary_codec_sv2;
use binary_sv2::{Deserialize, Serialize, B032};
#[cfg(not(feature = "with_serde"))]
use core::convert::TryInto;

/// Used by upstream to changes downstream node’s extranonce prefix.
///
/// This message will affect all jobs sent after this message on a given channel, provided by
/// upstream or introduced by [`SetCustomMiningJob`] message.
///
/// Note that this message is applicable only for opened standard or extended channels, not group
/// channels.
///
/// [`SetCustomMiningJob`]: crate::SetCustomMiningJob
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetExtranoncePrefix<'decoder> {
    /// Extended or standard channel identifier.
    pub channel_id: u32,
    /// Bytes used as implicit first part of extranonce.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub extranonce_prefix: B032<'decoder>,
}
#[cfg(feature = "with_serde")]
use binary_sv2::GetSize;
#[cfg(feature = "with_serde")]
impl<'d> GetSize for SetExtranoncePrefix<'d> {
    fn get_size(&self) -> usize {
        self.channel_id.get_size() + self.extranonce_prefix.get_size()
    }
}
#[cfg(feature = "with_serde")]
impl<'a> SetExtranoncePrefix<'a> {
    pub fn into_static(self) -> SetExtranoncePrefix<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
    pub fn as_static(&self) -> SetExtranoncePrefix<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
}

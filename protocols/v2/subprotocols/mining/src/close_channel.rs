#[cfg(not(feature = "with_serde"))]
use alloc::vec::Vec;
#[cfg(not(feature = "with_serde"))]
use binary_sv2::binary_codec_sv2;
use binary_sv2::{Deserialize, Serialize, Str0255};
#[cfg(not(feature = "with_serde"))]
use core::convert::TryInto;

/// Use by downstream role to announce end of operation.
///
/// A proxy must send this message on behalf of all opened channels from a downstream connection in
/// case of downstream connection closure.
///
/// Upon receiving this message, upstream must stop sending messages for the channel.
///
/// If a proxy is operating in channel aggregating mode (translating downstream channels into
/// aggregated extended upstream channels), it must send an [`UpdateChannel`] message when it
/// receives [`CloseChannel`] or connection closure from a downstream connection. In general, a
/// proxy must keep the upstream node notified about the real state of the downstream channels.
///
/// [`UpdateChannel`]: crate::UpdateChannel
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloseChannel<'decoder> {
    /// Channel id of the channel to be closed.
    pub channel_id: u32,
    /// Reason for closing the channel.
    #[cfg_attr(feature = "with_serde", serde(borrow))]
    pub reason_code: Str0255<'decoder>,
}

#[cfg(feature = "with_serde")]
use binary_sv2::GetSize;
#[cfg(feature = "with_serde")]
impl<'d> GetSize for CloseChannel<'d> {
    fn get_size(&self) -> usize {
        self.channel_id.get_size() + self.reason_code.get_size()
    }
}

#[cfg(feature = "with_serde")]
impl<'a> CloseChannel<'a> {
    pub fn into_static(self) -> CloseChannel<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
    pub fn as_static(&self) -> CloseChannel<'static> {
        panic!("This function shouldn't be called by the Message Generator");
    }
}

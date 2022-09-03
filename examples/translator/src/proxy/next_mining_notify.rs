use crate::downstream_sv1;
use async_channel::Sender;
use roles_logic_sv2::mining_sv2::{NewExtendedMiningJob, SetNewPrevHash};
use std::convert::TryInto;
use v1::{
    json_rpc, server_to_client,
    utils::{HexBytes, HexU32Be, PrevHash},
};

#[derive(Clone, Debug)]
pub struct NextMiningNotify {
    pub set_new_prev_hash: Option<SetNewPrevHash<'static>>,
    pub new_extended_mining_job: Option<NewExtendedMiningJob<'static>>,
    // pub sender_mining_notify: Sender<server_to_client::Notify>,
}

impl NextMiningNotify {
    pub(crate) fn new() -> Self {
        NextMiningNotify {
            set_new_prev_hash: None,
            new_extended_mining_job: None,
        }
    }

    /// Sets `set_new_prev_hash` member field upon `Bridge` receiving a SV2 `SetNewPrevHash`
    /// message from `Upstream`. Used in conjunction with `NewExtendedMiningJob` to create a SV1
    /// `mining.notify` message.
    pub(crate) fn set_new_prev_hash_msg(&mut self, set_new_prev_hash: SetNewPrevHash<'static>) {
        self.set_new_prev_hash = Some(set_new_prev_hash);
    }

    /// Sets `new_extended_mining_job` member field upon `Bridge` receiving a SV2
    /// `NewExtendedMiningJob` message from `Upstream`. Used in conjunction with `SetNewPrevHash`
    /// to create a SV1 `mining.notify` message.
    pub(crate) fn new_extended_mining_job_msg(
        &mut self,
        new_extended_mining_job: NewExtendedMiningJob<'static>,
    ) {
        self.new_extended_mining_job = Some(new_extended_mining_job);
    }

    /// `mining.notify`:  subscription id
    /// extranonce1
    /// extranonce_size2
    pub(crate) fn create_subscribe_response(&self) -> json_rpc::Message {
        println!("IN NEW_MINING_NOTIFY: {:?}", &self);
        let extra_nonce1 = downstream_sv1::new_extranonce();
        // let extranonce1_str: String = extra_nonce1.try_into().unwrap();
        let extra_nonce2_size = downstream_sv1::new_extranonce2_size();
        let difficulty = downstream_sv1::new_difficulty();
        let difficulty: String = difficulty.try_into().unwrap();
        let set_difficulty_str = format!("[\"mining.set_difficulty\", \"{}\"]", difficulty);
        let subscription_id = downstream_sv1::new_subscription_id();
        let subscription_id: String = subscription_id.try_into().unwrap();
        let notify_str = format!("[\"mining.notify\", \"{}\"]", subscription_id);
        let id = "1".to_string();
        let subscriptions = vec![(set_difficulty_str, notify_str)];

        let subscribe_response = server_to_client::Subscribe {
            id,
            extra_nonce1,
            extra_nonce2_size,
            subscriptions,
        };
        subscribe_response.into()
    }

    pub(crate) fn create_notify(&self) -> Option<server_to_client::Notify> {
        // Put logic in to make sure that SetNewPrevHash + NewExtendedMiningJob is matching (not
        // future)
        // if new_prev_hash.job_id != new_job.job_id {
        //     panic!("TODO: SetNewPrevHash + NewExtendedMiningJob job id's do not match");
        // }

        if self.set_new_prev_hash.is_some() && self.new_extended_mining_job.is_some() {
            println!("\nRR CREATE_NOTIFY\n");
            let new_prev_hash = match &self.set_new_prev_hash {
                Some(nph) => nph,
                None => panic!("Should never happen because of if statement"),
            };
            let new_job = match &self.new_extended_mining_job {
                Some(nj) => nj,
                None => panic!("Should never happen because of if statement"),
            };

            let job_id = new_prev_hash.job_id.to_string();

            // U256<'static> -> PrevHash
            let prev_hash_u256 = &new_prev_hash.prev_hash;
            let prev_hash_vec: Vec<u8> = prev_hash_u256.to_vec();
            let prev_hash = PrevHash(prev_hash_vec);

            // B064K<'static'> -> Vec<u8> -> String -> HexBytes
            let coin_base1_b064k = &new_job.coinbase_tx_prefix;
            let mut coin_base1_vec: Vec<u8> = coin_base1_b064k.to_vec();
            let coin_base1_slice: &[u8] = coin_base1_vec.as_mut_slice();
            // TODO: Check endianness
            let coin_base1_str = std::str::from_utf8(coin_base1_slice).unwrap();
            let coin_base1: HexBytes = coin_base1_str.try_into().unwrap();

            let coin_base2_b064k = &new_job.coinbase_tx_suffix;
            let mut coin_base2_vec: Vec<u8> = coin_base2_b064k.to_vec();
            let coin_base2_slice: &[u8] = coin_base2_vec.as_mut_slice();
            // TODO: Check endianness
            let coin_base2_str = std::str::from_utf8(coin_base2_slice).unwrap();
            let coin_base2: HexBytes = coin_base2_str.try_into().unwrap();

            // Seq0255<'static, U56<'static>> -> Vec<Vec<u8>> -> Vec<HexBytes>
            let merkle_path_seq0255 = &new_job.merkle_path;
            let merkle_path_vec = merkle_path_seq0255.clone().into_static();
            let merkle_path_vec: Vec<Vec<u8>> = merkle_path_vec.to_vec();
            let mut merkle_branch = Vec::<HexBytes>::new();
            // path: Vec<u8>
            for mut path in merkle_path_vec {
                let path_slice: &[u8] = path.as_mut_slice();
                // TODO: Check endianness
                let path_str = std::str::from_utf8(path_slice).unwrap();
                merkle_branch.push(path_str.try_into().unwrap());
            }

            // u32 -> String -> &str -> HexU32Be
            let version_u32 = new_job.version;
            let version_hex_str: &str = &format!("{:x}", version_u32);
            // TODO: Check endianness
            let version: HexU32Be = version_hex_str.try_into().unwrap();

            // u32 -> String -> &str -> HexU32Be
            let bits_u32 = new_prev_hash.nbits;
            let bits_hex_str: &str = &format!("{:x}", bits_u32);
            // TODO: Check endianness
            let bits: HexU32Be = bits_hex_str.try_into().unwrap();

            // u32 -> String -> &str -> HexU32Be
            let time_u32 = new_prev_hash.min_ntime;
            let time_hex_str: &str = &format!("{:x}", time_u32);
            // TODO: Check endianness
            let time: HexU32Be = time_hex_str.try_into().unwrap();

            let clean_jobs = false; // TODO: ?

            let notify_response = server_to_client::Notify {
                job_id,
                prev_hash,
                coin_base1,
                coin_base2,
                merkle_branch,
                version,
                bits,
                time,
                clean_jobs,
            };
            Some(notify_response)
        } else {
            None
        }
    }
}

use binary_sv2::U256;
use serde_json::{
    Value,
    Value::{Array as JArrary, Bool as JBool, Number as JNumber, String as JString},
};
use serde::{Serialize, Deserialize};
use std::convert::{TryFrom, TryInto};

use crate::{
    error::Error,
    json_rpc::{Message, Notification, Response},
    methods::ParsingMethodError,
    utils::{Extranonce, HexBytes, HexU32Be, MerkleNode, PrevHash},
};

// client.get_version()

// client.reconnect

// client.show_message

/// Fields in order:
///
/// * Job ID: This is included when miners submit a results so work can be matched with proper
///   transactions.
/// * Hash of previous block: Used to build the header.
/// * Generation transaction (part 1): The miner inserts ExtraNonce1 and ExtraNonce2 after this
///   section of the transaction data.
/// * Generation transaction (part 2): The miner appends this after the first part of the
///   transaction data and the two ExtraNonce values.
/// * List of merkle branches: The generation transaction is hashed against the merkle branches to
///   build the final merkle root.
/// * Bitcoin block version: Used in the block header.
///     * nBits: The encoded network difficulty. Used in the block header.
///     * nTime: The current time. nTime rolling should be supported, but should not increase faster
///     than actual time.
/// * Clean Jobs: If true, miners should abort their current work and immediately use the new job.
///   If false, they can still use the current job, but should move to the new one after exhausting
///   the current nonce range.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notify<'a> {
    pub job_id: String,
    #[serde(borrow)]
    pub prev_hash: PrevHash<'a>,
    pub coin_base1: HexBytes,
    pub coin_base2: HexBytes,
    #[serde(borrow)]
    pub merkle_branch: Vec<MerkleNode<'a>>,
    pub version: HexU32Be,
    pub bits: HexU32Be,
    pub time: HexU32Be,
    pub clean_jobs: bool,
}

impl<'a> TryFrom<Notify<'a>> for Message {
    type Error = Error<'a>;

    fn try_from(notify: Notify) -> Result<Self, Error> {
        let prev_hash: Value = notify.prev_hash.try_into()?;
        let coin_base1: Value = notify.coin_base1.try_into()?;
        let coin_base2: Value = notify.coin_base2.try_into()?;
        let mut merkle_branch: Vec<Value> = vec![];
        for mb in notify.merkle_branch {
            let mb: Value = mb.try_into()?;
            merkle_branch.push(mb);
        }
        let merkle_branch = JArrary(merkle_branch);
        let version: Value = notify.version.try_into()?;
        let bits: Value = notify.bits.try_into()?;
        let time: Value = notify.time.try_into()?;
        Ok(Message::Notification(Notification {
            method: "mining.notify".to_string(),
            params: (&[
                notify.job_id.into(),
                prev_hash,
                coin_base1,
                coin_base2,
                merkle_branch,
                version,
                bits,
                time,
                notify.clean_jobs.into(),
            ][..])
                .into(),
        }))
    }
}

impl<'a> TryFrom<Notification> for Notify<'a> {
    type Error = ParsingMethodError;

    #[allow(clippy::many_single_char_names)]
    fn try_from(msg: Notification) -> Result<Self, Self::Error> {
        let params = msg
            .params
            .as_array()
            .ok_or_else(|| ParsingMethodError::not_array_from_value(msg.params.clone()))?;
        let (
            job_id,
            prev_hash,
            coin_base1,
            coin_base2,
            merkle_branch_,
            version,
            bits,
            time,
            clean_jobs,
        ) = match &params[..] {
            [JString(a), JString(b), JString(c), JString(d), JArrary(e), JString(f), JString(g), JString(h), JBool(i)] => {
                (
                    a.into(),
                    b.as_str().try_into()?,
                    c.as_str().try_into()?,
                    d.as_str().try_into()?,
                    e,
                    f.as_str().try_into()?,
                    g.as_str().try_into()?,
                    h.as_str().try_into()?,
                    *i,
                )
            }
            _ => {
                return Err(ParsingMethodError::wrong_args_from_value(
                    msg.params.clone(),
                ))
            }
        };
        let mut merkle_branch = vec![];
        for h in merkle_branch_ {
            let h: MerkleNode = h
                .as_str()
                .ok_or_else(|| ParsingMethodError::not_string_from_value(h.clone()))?
                .try_into()?;

            merkle_branch.push(h);
        }
        let notify = Notify {
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
        Ok(notify.clone())
    }
}

/// mining.set_difficulty(difficulty)
///
/// The server can adjust the difficulty required for miner shares with the "mining.set_difficulty"
/// method. The miner should begin enforcing the new difficulty on the next job received. Some pools
/// may force a new job out when set_difficulty is sent, using clean_jobs to force the miner to
/// begin using the new difficulty immediately.
///
#[derive(Debug)]
pub struct SetDifficulty {
    pub value: f64,
}

impl From<SetDifficulty> for Message {
    fn from(sd: SetDifficulty) -> Self {
        let value: Value = sd.value.into();
        Message::Notification(Notification {
            method: "mining.set_difficulty".to_string(),
            params: (&[value][..]).into(),
        })
    }
}

impl TryFrom<Notification> for SetDifficulty {
    type Error = ParsingMethodError;

    fn try_from(msg: Notification) -> Result<Self, Self::Error> {
        let params = msg
            .params
            .as_array()
            .ok_or_else(|| ParsingMethodError::not_array_from_value(msg.params.clone()))?;
        let (value,) = match &params[..] {
            [a] => (a
                .as_f64()
                .ok_or_else(|| ParsingMethodError::not_float_from_value(a.clone()))?,),
            _ => return Err(ParsingMethodError::wrong_args_from_value(msg.params)),
        };
        Ok(SetDifficulty { value })
    }
}

/// SetExtranonce message (sent if we subscribed with `ExtranonceSubscribe`)
///
/// mining.set_extranonce("extranonce1", extranonce2_size)
///
/// These values, when provided, replace the initial subscription values beginning with the next
/// mining.notify job.
///
/// check if it is a Notification or a StandardRequest this implementation assume that it is a
/// Notification
///
#[derive(Debug)]
pub struct SetExtranonce<'a> {
    pub extra_nonce1: Extranonce<'a>,
    pub extra_nonce2_size: usize,
}

impl<'a> TryFrom<SetExtranonce<'a>> for Message {
    type Error = Error<'a>;

    fn try_from(se: SetExtranonce) -> Result<Self, Error> {
        let extra_nonce1: Value = se.extra_nonce1.try_into()?;
        let extra_nonce2_size: Value = se.extra_nonce2_size.into();
        Ok(Message::Notification(Notification {
            method: "mining.set_extranonce".to_string(),
            params: (&[extra_nonce1, extra_nonce2_size][..]).into(),
        }))
    }
}

impl<'a> TryFrom<Notification> for SetExtranonce<'a> {
    type Error = ParsingMethodError;

    fn try_from(msg: Notification) -> Result<Self, Self::Error> {
        let params = msg
            .params
            .as_array()
            .ok_or_else(|| ParsingMethodError::not_array_from_value(msg.params.clone()))?;
        let (extra_nonce1, extra_nonce2_size) = match &params[..] {
            [JString(a), JNumber(b)] => (
                Extranonce::try_from(hex::decode(a)?)?,
                b.as_u64()
                    .ok_or_else(|| ParsingMethodError::not_unsigned_from_value(b.clone()))?
                    as usize,
            ),
            _ => return Err(ParsingMethodError::wrong_args_from_value(msg.params)),
        };
        Ok(SetExtranonce {
            extra_nonce1,
            extra_nonce2_size,
        })
    }
}

#[derive(Debug)]
/// Server may arbitrarily adjust version mask
pub struct SetVersionMask {
    version_mask: HexU32Be,
}

impl TryFrom<SetVersionMask> for Message {
    type Error = Error<'static>;

    fn try_from(sv: SetVersionMask) -> Result<Self, Error<'static>> {
        let version_mask: Value = sv.version_mask.try_into()?;
        Ok(Message::Notification(Notification {
            method: "mining.set_version".to_string(),
            params: (&[version_mask][..]).into(),
        }))
    }
}

impl TryFrom<Notification> for SetVersionMask {
    type Error = ParsingMethodError;

    fn try_from(msg: Notification) -> Result<Self, Self::Error> {
        let params = msg
            .params
            .as_array()
            .ok_or_else(|| ParsingMethodError::not_array_from_value(msg.params.clone()))?;
        let version_mask = match &params[..] {
            [JString(a)] => a.as_str().try_into()?,
            _ => return Err(ParsingMethodError::wrong_args_from_value(msg.params)),
        };
        Ok(SetVersionMask { version_mask })
    }
}

//pub struct Authorize(pub crate::json_rpc::Response, pub String);

/// Authorize and Submit responsed are identical
#[derive(Debug, Clone)]
pub struct GeneralResponse {
    pub id: u64,
    result: bool,
}

impl GeneralResponse {
    pub fn into_authorize(self, prev_request_name: String) -> Authorize {
        Authorize {
            id: self.id,
            authorized: self.result,
            prev_request_name,
        }
    }
    pub fn into_submit(self) -> Submit {
        Submit {
            id: self.id,
            is_ok: self.result,
        }
    }
}

impl TryFrom<&Response> for GeneralResponse {
    type Error = ParsingMethodError;

    fn try_from(msg: &Response) -> Result<Self, Self::Error> {
        let id = msg.id;
        let result = msg.result.as_bool().ok_or_else(|| {
            ParsingMethodError::ImpossibleToParseResultField(Box::new(msg.clone()))
        })?;
        Ok(GeneralResponse { id, result })
    }
}

#[derive(Debug, Clone)]
pub struct Authorize {
    pub id: u64,
    authorized: bool,
    pub prev_request_name: String,
}

impl Authorize {
    pub fn is_ok(&self) -> bool {
        self.authorized
    }

    pub fn user_name(self) -> String {
        self.prev_request_name
    }
}

#[derive(Debug)]
pub struct Submit {
    pub id: u64,
    is_ok: bool,
}

impl Submit {
    pub fn is_ok(&self) -> bool {
        self.is_ok
    }
}

/// mining.subscribe
/// mining.subscribe("user agent/version", "extranonce1")
/// The optional second parameter specifies a mining.notify subscription id the client wishes to resume
/// working with (possibly due to a dropped connection). If provided, a server MAY (at its option)
/// issue the connection the same extranonce1. Note that the extranonce1 may be the same (allowing
/// a resumed connection) even if the subscription id is changed!
///
/// The client receives a result:
///
///
/// The result contains three items:
///
///    Subscriptions. - An array of 2-item tuples, each with a subscription type and id.
///
///    ExtraNonce1. - Hex-encoded, per-connection unique string which will be used for creating
///    generation transactions later.
///
///    ExtraNonce2_size. - The number of bytes that the miner users for its ExtraNonce2 counter.
///
#[derive(Debug)]
pub struct Subscribe<'a> {
    pub id: u64,
    pub extra_nonce1: Extranonce<'a>,
    pub extra_nonce2_size: usize,
    pub subscriptions: Vec<(String, String)>,
}

impl<'a> From<Subscribe<'a>> for Message {
    fn from(su: Subscribe) -> Self {
        let extra_nonce1: Value = su.extra_nonce1.into();
        let extra_nonce2_size: Value = su.extra_nonce2_size.into();
        let subscriptions: Vec<Value> = su
            .subscriptions
            .iter()
            .map(|x| JArrary(vec![JString(x.0.clone()), JString(x.1.clone())]))
            .collect();
        let subscriptions: Value = subscriptions.into();
        Message::OkResponse(Response {
            id: su.id,
            error: None,
            result: (&[subscriptions, extra_nonce1, extra_nonce2_size][..]).into(),
        })
    }
}

impl<'a> TryFrom<&Response> for Subscribe<'a> {
    type Error = ParsingMethodError;

    fn try_from(msg: &Response) -> Result<Self, Self::Error> {
        let id = msg.id;
        let params = msg.result.as_array().ok_or_else(|| {
            ParsingMethodError::ImpossibleToParseResultField(Box::new(msg.clone()))
        })?;
        let (subscriptions_, extra_nonce1, extra_nonce2_size) = match &params[..] {
            [JArrary(a), JString(b), JNumber(c)] => (
                a,
                // infallible
                b.as_str().try_into()?,
                c.as_u64().ok_or_else(|| {
                    ParsingMethodError::ImpossibleToParseAsU64(Box::new(c.clone()))
                })? as usize,
            ),
            _ => return Err(ParsingMethodError::UnexpectedArrayParams(params.clone())),
        };
        let mut subscriptions: Vec<(String, String)> = vec![];
        for s in subscriptions_ {
            // we already checked that subscriptions_ is an array
            let s = s.as_array().unwrap();
            if s.len() != 2 {
                return Err(ParsingMethodError::UnexpectedArrayParams(params.clone()));
            };
            let s = (
                s[0].as_str()
                    .ok_or_else(|| ParsingMethodError::UnexpectedArrayParams(params.clone()))?
                    .to_string(),
                s[1].as_str()
                    .ok_or_else(|| ParsingMethodError::UnexpectedArrayParams(params.clone()))?
                    .to_string(),
            );
            subscriptions.push(s);
        }
        Ok(Subscribe {
            id,
            extra_nonce1,
            extra_nonce2_size,
            subscriptions,
        })
    }
}

#[derive(Debug)]
pub struct Configure {
    pub id: u64,
    pub version_rolling: Option<VersionRollingParams>,
    pub minimum_difficulty: Option<bool>,
}

impl Configure {
    pub fn version_rolling_mask(&self) -> Option<HexU32Be> {
        match &self.version_rolling {
            None => None,
            Some(a) => {
                if a.version_rolling {
                    Some(a.version_rolling_mask.clone())
                } else {
                    None
                }
            }
        }
    }
    pub fn version_rolling_min_bit(&self) -> Option<HexU32Be> {
        match &self.version_rolling {
            None => None,
            Some(a) => {
                if a.version_rolling {
                    Some(a.version_rolling_min_bit_count.clone())
                } else {
                    None
                }
            }
        }
    }
}

impl From<Configure> for Message {
    fn from(co: Configure) -> Self {
        let mut params = serde_json::Map::new();
        if let Some(version_rolling_) = co.version_rolling {
            let mut version_rolling: serde_json::Map<String, Value> =
                // infallible
                version_rolling_.try_into().unwrap();
            params.append(&mut version_rolling);
        };
        if let Some(min_diff) = co.minimum_difficulty {
            let minimum_difficulty: Value = min_diff.into();
            params.insert("minimum-difficulty".to_string(), minimum_difficulty);
        };
        Message::OkResponse(Response {
            id: co.id,
            error: None,
            result: serde_json::Value::Object(params),
        })
    }
}

impl TryFrom<&Response> for Configure {
    type Error = ParsingMethodError;

    fn try_from(msg: &Response) -> Result<Self, ParsingMethodError> {
        let id = msg.id;
        let params = msg.result.as_object().ok_or_else(|| {
            ParsingMethodError::ImpossibleToParseResultField(Box::new(msg.clone()))
        })?;

        let version_rolling_ = params.get("version-rolling");
        let version_rolling_mask = params.get("version-rolling.mask");
        let version_rolling_min_bit_count = params.get("version-rolling.min-bit-count");
        let minimum_difficulty = params.get("minimum-difficulty");

        // Deserialize version-rolling response.
        // Composed by 3 fields:
        //   version-rolling (required),
        //   version-rolling.mask (required)
        //   version-rolling.min-bit-count (optional)
        let version_rolling: Option<VersionRollingParams>;
        if version_rolling_.is_some() && version_rolling_mask.is_some() {
            let vr: bool = version_rolling_
                .unwrap()
                .as_bool()
                .ok_or_else(|| ParsingMethodError::UnexpectedObjectParams(params.clone()))?;

            let version_rolling_mask: HexU32Be = version_rolling_mask
                .unwrap()
                .as_str()
                .ok_or_else(|| ParsingMethodError::UnexpectedObjectParams(params.clone()))?
                .try_into()?;

            // version-rolling.min-bit-count is often not returned by stratum servers,
            // but min-bit-count should be taken into consideration in the returned mask
            let version_rolling_min_bit_count: HexU32Be = match version_rolling_min_bit_count {
                Some(version_rolling_min_bit_count) => version_rolling_min_bit_count
                    .as_str()
                    .ok_or_else(|| ParsingMethodError::UnexpectedObjectParams(params.clone()))?
                    .try_into()?,
                None => HexU32Be(0),
            };

            version_rolling = Some(VersionRollingParams {
                version_rolling: vr,
                version_rolling_mask,
                version_rolling_min_bit_count,
            });
        } else if version_rolling_.is_none()
            && version_rolling_mask.is_none()
            && version_rolling_min_bit_count.is_none()
        {
            version_rolling = None;
        } else {
            return Err(ParsingMethodError::UnexpectedObjectParams(params.clone()));
        };

        let minimum_difficulty = match minimum_difficulty {
            Some(a) => Some(
                a.as_bool()
                    .ok_or_else(|| ParsingMethodError::UnexpectedObjectParams(params.clone()))?,
            ),
            None => None,
        };

        Ok(Configure {
            id,
            version_rolling,
            minimum_difficulty,
        })
    }
}

#[derive(Debug)]
pub struct VersionRollingParams {
    pub version_rolling: bool,
    pub version_rolling_mask: HexU32Be,
    pub version_rolling_min_bit_count: HexU32Be,
}

#[test]
fn configure_response_parsing_all_fields() {
    let client_response_str = r#"{"id":0,
            "result":{
                "version-rolling":true,
                "version-rolling.mask":"1fffe000",
                "version-rolling.min-bit-count":"00000005",
                "minimum-difficulty":false
            }
        }"#;
    let client_response = serde_json::from_str(&client_response_str).unwrap();
    let server_configure = Configure::try_from(&client_response).unwrap();
    println!("{:?}", server_configure);

    let version_rolling = server_configure.version_rolling.unwrap();
    assert_eq!(version_rolling.version_rolling, true);
    assert_eq!(version_rolling.version_rolling_mask, HexU32Be(0x1fffe000));
    assert_eq!(version_rolling.version_rolling_min_bit_count, HexU32Be(5));

    assert_eq!(server_configure.minimum_difficulty, Some(false));
}

#[test]
fn configure_response_parsing_no_vr_min_bit_count() {
    let client_response_str = r#"{"id":0,
            "result":{
                "version-rolling":true,
                "version-rolling.mask":"1fffe000",
                "minimum-difficulty":false
            }
        }"#;
    let client_response = serde_json::from_str(&client_response_str).unwrap();
    let server_configure = Configure::try_from(&client_response).unwrap();
    println!("{:?}", server_configure);

    let version_rolling = server_configure.version_rolling.unwrap();
    assert_eq!(version_rolling.version_rolling, true);
    assert_eq!(version_rolling.version_rolling_mask, HexU32Be(0x1fffe000));
    assert_eq!(version_rolling.version_rolling_min_bit_count, HexU32Be(0));

    assert_eq!(server_configure.minimum_difficulty, Some(false));
}

impl VersionRollingParams {
    pub fn new(
        version_rolling_mask: HexU32Be,
        version_rolling_min_bit_count: HexU32Be,
    ) -> Result<Self, Error<'static>> {
        // 0x1FFFE000 should be configured
        let negotiated_mask = HexU32Be(version_rolling_mask.clone() & 0x1FFFE000);

        let version_head_ok = negotiated_mask.0 >> 29 == 0;
        let version_tail_ok = negotiated_mask.0 << 19 == 0;
        if version_head_ok && version_tail_ok {
            Ok(VersionRollingParams {
                version_rolling: true,
                version_rolling_mask: negotiated_mask,
                version_rolling_min_bit_count,
            })
        } else {
            Err(Error::InvalidVersionMask(version_rolling_mask))
        }
    }
}

impl TryFrom<VersionRollingParams> for serde_json::Map<String, Value> {
    type Error = Error<'static>;

    fn try_from(vp: VersionRollingParams) -> Result<Self, Error<'static>> {
        let version_rolling: Value = vp.version_rolling.into();
        let version_rolling_mask: Value = vp.version_rolling_mask.try_into()?;
        let version_rolling_min_bit_count: Value = vp.version_rolling_min_bit_count.try_into()?;
        let mut params = serde_json::Map::new();
        params.insert("version-rolling".to_string(), version_rolling);
        params.insert("version-rolling.mask".to_string(), version_rolling_mask);
        params.insert(
            "version-rolling.min-bit-count".to_string(),
            version_rolling_min_bit_count,
        );
        Ok(params)
    }
}

#[test]
fn test_notify_serialization(){
    let server_message = r#"{"job_id":"4f","prev_hash":"4d16b6f85af6e2198f44ae2a6de67f78","coin_base1":"01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff20020862062f503253482f04b8864e5008","coin_base2":"072f736c7573682f000000000100f2052a010000001976a914d23fcdf86f7e756a64a7a9688ef9903327048ed988ac00000000","merkle_branch":["4d16b6f85af6e2198f44ae2a6de67f78"],"version":"00000002","bits":"1c2ac4af","time":"504e86b9","clean_jobs":false}"#;
    let notify: Notify = Notify { 
        job_id: "4f".to_string(), 
        prev_hash: PrevHash(U256::try_from([52, 100, 49, 54, 98, 54, 102, 56, 53, 97, 102, 54, 101, 50, 49, 57, 56, 102, 52, 52, 97, 101, 50, 97, 54, 100, 101, 54, 55, 102, 55, 56].to_vec()).unwrap()), 
        coin_base1: HexBytes::from(
            [48, 49, 48, 48, 48, 48, 48, 48, 48, 49, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 102, 102, 102, 102, 102, 102, 102, 102, 50, 48, 48, 50, 48, 56, 54, 50, 48, 54, 50, 102, 53, 48, 51, 50, 53, 51, 52, 56, 50, 102, 48, 52, 98, 56, 56, 54, 52, 101, 53, 48, 48, 56].to_vec()), 
        coin_base2: HexBytes::from([48, 55, 50, 102, 55, 51, 54, 99, 55, 53, 55, 51, 54, 56, 50, 102, 48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 48, 48, 102, 50, 48, 53, 50, 97, 48, 49, 48, 48, 48, 48, 48, 48, 49, 57, 55, 54, 97, 57, 49, 52, 100, 50, 51, 102, 99, 100, 102, 56, 54, 102, 55, 101, 55, 53, 54, 97, 54, 52, 97, 55, 97, 57, 54, 56, 56, 101, 102, 57, 57, 48, 51, 51, 50, 55, 48, 52, 56, 101, 100, 57, 56, 56, 97, 99, 48, 48, 48, 48, 48, 48, 48, 48].to_vec()), 
        merkle_branch: [MerkleNode(U256::try_from([52, 100, 49, 54, 98, 54, 102, 56, 53, 97, 102, 54, 101, 50, 49, 57, 56, 102, 52, 52, 97, 101, 50, 97, 54, 100, 101, 54, 55, 102, 55, 56].to_vec()).unwrap())].to_vec(), 
        version: HexU32Be(2), 
        bits: HexU32Be(472564911), 
        time: HexU32Be(1347323577), 
        clean_jobs: false };
    let serialized_string = serde_json::to_string(&notify).unwrap();
    assert_eq!(server_message, serialized_string);
}

#[test]
fn test_notify_serde(){
    let server_message = r#"{"job_id":"4f","prev_hash":"4d16b6f85af6e2198f44ae2a6de67f78","coin_base1":"01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff20020862062f503253482f04b8864e5008","coin_base2":"072f736c7573682f000000000100f2052a010000001976a914d23fcdf86f7e756a64a7a9688ef9903327048ed988ac00000000","merkle_branch":["4d16b6f85af6e2198f44ae2a6de67f78"],"version":"00000002","bits":"1c2ac4af","time":"504e86b9","clean_jobs":false}"#;
    let notify: Notify = serde_json::from_str(&server_message).unwrap();
    let serialized_message = serde_json::to_string(&notify).unwrap();
    assert_eq!(server_message, serialized_message);
}
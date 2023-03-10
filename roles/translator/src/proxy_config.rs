
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProxyConfig {
    pub upstream_address: String,
    pub upstream_port: u16,
    pub upstream_authority_pubkey: String,
    pub downstream_address: String,
    pub downstream_port: u16,
    pub max_supported_version: u16,
    pub min_supported_version: u16,
    pub min_extranonce2_size: u16,
    pub jn_config: Option<JnConfig>,
}

#[derive(Debug, Deserialize)]
pub struct JnConfig {
    pub jn_address: String,
    pub tp_address: String,
}
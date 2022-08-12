use crate::primitives::{Amount};
use serde;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "lowercase")]
pub enum HoleRequest {
    SetChannel(SetChannelRequest)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SetChannelRequest {
    #[serde(alias = "id")]
    pub id: String,
    #[serde(alias = "feebase", skip_serializing_if = "Option::is_none")]
    pub fee_base_msat: Option<Amount>,
    #[serde(alias = "feeppm", skip_serializing_if = "Option::is_none")]
    pub fee_ppm: Option<u64>,
    #[serde(alias = "htlcmin", skip_serializing_if = "Option::is_none")]
    pub htlc_min_msat: Option<Amount>,
    #[serde(alias = "htlcmax", skip_serializing_if = "Option::is_none")]
    pub htlc_max_msat: Option<Amount>,
    #[serde(alias = "enforcedelay", skip_serializing_if = "Option::is_none")]
    pub enforce_delay: Option<u32>,
}

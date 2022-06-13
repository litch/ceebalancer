use std::path::{Path};
use serde::{Deserialize, Serialize};

use crate::primitives::{Amount, Address};

#[derive(Debug, Deserialize)]
pub struct GetInfoResponse {
    method: String,
    pub result: GetInfoResponseInfo
}

#[derive(Debug, Deserialize)]
pub struct GetInfoResponseInfo {
}

#[derive(Debug, Deserialize)]
pub struct ListChannelsResponse {
    method: String,
    pub result: ListChannelsResponseChannels
}

#[derive(Debug, Deserialize)]
pub struct ListChannelsResponseChannels {
    pub channels: Vec<ListChannel>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListChannel {
    pub source: String,
    pub destination: String,
    pub short_channel_id: String,
    pub amount_msat: Amount,
}

#[derive(Debug, Deserialize)]
pub struct ListNodesResponse {
    pub method: String,
    pub result: ListNodesResponseNodes
}

#[derive(Debug, Deserialize)]
pub struct ListNodesResponseNodes {
    pub nodes: Vec<ListNode>
}

#[derive(Debug, Deserialize)]
pub struct ListNode {
    pub nodeid: String,
    pub addresses: Vec<Address>,
    pub last_timestamp: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_will_fund: Option<OptionWillFund>
}

#[derive(Debug, Deserialize)]
pub struct OptionWillFund {
    pub lease_fee_base_msat: Amount,
    pub lease_fee_basis: u32,
    pub funding_weight: u32,
    pub channel_fee_max_base_msat: Amount,
    pub channel_fee_max_proportional_thousandths: u32,
    pub compact_lease: String,
}
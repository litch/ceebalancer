use cln_rpc::{
    model,
    ClnRpc, 
    Request,
    Response,
};
use std::path::{Path};

use reqwest;
use anyhow::{anyhow, Error};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

pub mod primitives;
pub mod wire;

use std::sync::{Arc, RwLock};

#[derive(Default, Debug)]
pub struct Config {
    pub dynamic_fees: bool,
    pub dynamic_fee_min: i64,
    pub dynamic_fee_max: i64,
}

impl Config {
    pub fn default() -> Config {
        Config { 
            dynamic_fees: false, dynamic_fee_min: 0, dynamic_fee_max: 1000
        }
    }

    pub fn current() -> Arc<Config> {
        CURRENT_CONFIG.with(|c| c.read().unwrap().clone())
    }
    pub fn make_current(self) {
        CURRENT_CONFIG.with(|c| *c.write().unwrap() = Arc::new(self))
    }
}

thread_local! {
    static CURRENT_CONFIG: RwLock<Arc<Config>> = RwLock::new(Default::default());
}

pub async fn get_info() -> Result<String, Error> {
    let req = Request::Getinfo(model::GetinfoRequest {});
    // call(req).await?;
    Ok(call(req).await?)
}

pub async fn list_channels() -> Result<Vec<wire::ListChannel>, Error> {
    let req = Request::ListChannels(model::ListchannelsRequest { short_channel_id: None, source: None, destination: None });
    let res = call(req).await.unwrap();
    let de: wire::ListChannelsResponse = serde_json::from_str(&res).unwrap();
    
    Ok(de.result.channels)
    // dbg!(res);
}

pub async fn list_nodes() -> Result<(), Error> {
    let req = Request::ListNodes(model::ListnodesRequest {id: None});
    let res = call(req).await?;
    let de: wire::ListNodesResponse = serde_json::from_str(&res).unwrap();
    // dbg!(de);
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ListFundsResponse {
    method: String,
    result: ListFundsResponseFunds
}

#[derive(Debug, Deserialize)]
pub struct ListFundsResponseFunds {
    outputs: Vec<Output>
}

#[derive(Debug, Deserialize)]
pub struct Output {
    txid: String,
    amount_msat: primitives::Amount,
    status: String,
}


pub async fn onchain_balance() -> Result<u64, Error> {
    let req = Request::ListFunds(model::ListfundsRequest { spent: Some(false)});
    let res = call(req).await?;
    let de: ListFundsResponse = serde_json::from_str(&res).unwrap();

    let mut total = 0;
    for output in de.result.outputs {
        total += output.amount_msat.msat();
    }

    Ok(total)
}

pub async fn call(request: Request) -> Result<String, Error> {
    let path = Path::new("lightning-rpc");
    // dbg!(&path);

    let mut rpc = ClnRpc::new(path).await?;
    let response = rpc
        .call(request.clone())
        .await
        .map_err(|e| anyhow!("Error calling {:?}: {:?}", request, e))?;
    // dbg!(serde_json::to_string_pretty(&response)?);
    Ok(serde_json::to_string_pretty(&response)?)
}

pub async fn report_onchain(balance: u64) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:8877/report")
        .json(&serde_json::json!({
            "id": "some-node-id",
            "balance": balance}))
        .send()
        .await?;
    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;


    use serde_json::json;

    #[tokio::test]
    async fn test_list_channels() {
        let j = json!({
            "method": "listchannels",
            "result": {
              "channels": [
                {
                  "source": "025007779efdbc17b968bfc79a7fc72f0f82150e9402d9d9a6f745a18cb32b5dfa",
                  "destination": "035954e4f315fd0067fbc41a05bf2f35be6020ab67f047a1c46bac3126d0614574",
                  "short_channel_id": "121x1x0",
                  "public": true,
                  "amount_msat": "10000000000msat",
                  "message_flags": 1,
                  "channel_flags": 0,
                  "active": true,
                  "last_update": 1654271745,
                  "base_fee_millisatoshi": 1,
                  "fee_per_millionth": 10,
                  "delay": 6,
                  "htlc_minimum_msat": "0msat",
                  "htlc_maximum_msat": "9900000000msat",
                  "features": ""
                },
                {
                  "source": "035954e4f315fd0067fbc41a05bf2f35be6020ab67f047a1c46bac3126d0614574",
                  "destination": "025007779efdbc17b968bfc79a7fc72f0f82150e9402d9d9a6f745a18cb32b5dfa",
                  "short_channel_id": "121x1x0",
                  "public": true,
                  "amount_msat": "10000000000msat",
                  "message_flags": 1,
                  "channel_flags": 1,
                  "active": true,
                  "last_update": 1654271750,
                  "base_fee_millisatoshi": 1,
                  "fee_per_millionth": 10,
                  "delay": 6,
                  "htlc_minimum_msat": "0msat",
                  "htlc_maximum_msat": "9900000000msat",
                  "features": ""
                },
                {
                  "source": "025007779efdbc17b968bfc79a7fc72f0f82150e9402d9d9a6f745a18cb32b5dfa",
                  "destination": "0311921103cf410329837f07d2c1681ae882420c800e0020600040605afbd0f04c",
                  "short_channel_id": "124x2x1",
                  "public": true,
                  "amount_msat": "7000000000msat",
                  "message_flags": 1,
                  "channel_flags": 0,
                  "active": true,
                  "last_update": 1654271772,
                  "base_fee_millisatoshi": 1,
                  "fee_per_millionth": 10,
                  "delay": 6,
                  "htlc_minimum_msat": "0msat",
                  "htlc_maximum_msat": "6930000000msat",
                  "features": ""
                }]}});
        let de: ListChannelsResponse = serde_json::from_value(j).unwrap();
        assert_eq!(de.result.channels[0].amount_msat.msat(), 10000000000)
    }
}

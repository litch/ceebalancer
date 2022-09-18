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

pub async fn list_nodes() -> Result<(), Error> {
    let req = Request::ListNodes(model::ListnodesRequest {id: None});
    let res = call(req).await?;
    let de: wire::ListNodesResponse = serde_json::from_str(&res).unwrap();
    
    Ok(())
}

pub async fn list_channels() -> Result<Vec<wire::Channel>, Error> {
    let req = Request::ListFunds(model::ListfundsRequest { spent: Some(false)});
    let res = call(req).await?;
    log::debug!("{}", &res);

    let de: wire::ListFundsResponse = serde_json::from_str(&res).unwrap();
    
    Ok(de.result.channels)
}

pub async fn onchain_balance() -> Result<u64, Error> {
    let req = Request::ListFunds(model::ListfundsRequest { spent: Some(false)});
    let res = call(req).await?;
    let de: wire::ListFundsResponse = serde_json::from_str(&res).unwrap();

    let mut total = 0;
    for output in de.result.outputs {
        total += output.amount_msat.msat();
    }
    
    Ok(total)
}

pub async fn set_channel_fee(channel: wire::Channel, fee: u32) -> Result<(), Error> {
    let req = Request::SetChannel(model::SetchannelRequest {
        id: channel.short_channel_id.expect("Channel not ready yet"),
        feeppm: Some(fee),
        feebase: None,
        htlcmax: None,
        htlcmin: None,
    });
    let res = call(req).await?;
    log::info!("Set channel: {:?}", res);

    Ok(())
}

pub async fn call(request: Request) -> Result<String, Error> {
    let path = Path::new("lightning-rpc");
    let mut rpc = ClnRpc::new(path).await?;
    let response = rpc
        .call(request.clone())
        .await
        .map_err(|e| anyhow!("Error calling {:?}: {:?}", request, e))?;
    
    Ok(serde_json::to_string_pretty(&response)?)
}

pub async fn calculate_fee_target(channel: &wire::Channel) -> Result<u32, Error> {
    let ours: f64 = channel.our_amount_msat.msat() as f64; 
    let total: f64 = channel.amount_msat.msat() as f64;
    let proportion = 1.0 - (ours / total);

    let min_threshold_ratio = 0.2;
    let max_threshold_ratio = 0.8;

    let max: f64 = Config::current().dynamic_fee_max as f64;
    let min: f64 = Config::current().dynamic_fee_min as f64;

    let range = max - min;

    let target = if proportion <= min_threshold_ratio {
        min
    } else if proportion >= max_threshold_ratio {
        max
    } else { 
        let nom = proportion - min_threshold_ratio;
        let denom = max_threshold_ratio - min_threshold_ratio;
        ((nom / denom) * range) + min
    };
    
    Ok(target.round() as u32)
}


#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_calculate_balanced_channel() {
        Config {
            dynamic_fees: true,
            dynamic_fee_min: 10,
            dynamic_fee_max: 500,
        }.make_current();

        let c = wire::Channel {
            amount_msat: primitives::Amount {msat: 1000000},
            our_amount_msat: primitives::Amount {msat: 500000},
            connected: true,
            peer_id: "039b9e260863e6d8735325b286931d73be9f8e766970ad4fe1cbcc470cd8964635".to_string(),
            state: wire::ChannelState::CHANNELD_NORMAL,
            funding_txid: "724ee70bc1670368c3db3c2ebed30d00fa595774356cebf509196c68a471ca91".to_string(),
            funding_output: 0,
            short_channel_id: Some("123x123x0".to_string()),
        };

        let target = calculate_fee_target(c).await.unwrap();
        assert_eq!(target, 255)   
    }

    #[tokio::test]
    async fn calculate_imbalanced_channel() {
        Config {
            dynamic_fees: true,
            dynamic_fee_min: 10,
            dynamic_fee_max: 500,
        }.make_current();

        let test_cases = vec![
            (1000, 1000, 10),
            (1000, 0, 500),
            (1000, 200, 500),
            (1000, 205, 496),
            (1000, 795, 14)
        ];

        for (channel_size, ours, fee) in test_cases {
            let c = wire::Channel {
                amount_msat: primitives::Amount {msat: channel_size},
                our_amount_msat: primitives::Amount {msat: ours},
                connected: true,
                peer_id: "039b9e260863e6d8735325b286931d73be9f8e766970ad4fe1cbcc470cd8964635".to_string(),
                state: wire::ChannelState::CHANNELD_NORMAL,
                funding_txid: "724ee70bc1670368c3db3c2ebed30d00fa595774356cebf509196c68a471ca91".to_string(),
                funding_output: 0,
                short_channel_id: Some("123x123x0".to_string()),
            };
    
            let target = calculate_fee_target(c).await.unwrap();
            assert_eq!(target, fee)
        }
    }

    #[tokio::test]
    async fn test_list_funds() {
        let j = json!({
            "method": "listfunds",
            "result": {
                "outputs": [],
                "channels": [
                   {
                      "peer_id": "039b9e260863e6d8735325b286931d73be9f8e766970ad4fe1cbcc470cd8964635",
                      "connected": true,
                      "state": "CHANNELD_NORMAL",
                      "short_channel_id": "206x5x0",
                      "our_amount_msat": "4000000000msat",
                      "amount_msat": "4000000000msat",
                      "funding_txid": "724ee70bc1670368c3db3c2ebed30d00fa595774356cebf509196c68a471ca91",
                      "funding_output": 0
                   }
                ]
             }
        });
        let de: wire::ListFundsResponse = serde_json::from_value(j).unwrap();
        assert_eq!(de.result.channels[0].amount_msat.msat(), 4000000000)   
    }


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
        let de: wire::ListChannelsResponse = serde_json::from_value(j).unwrap();
        assert_eq!(de.result.channels[0].amount_msat.msat(), 10000000000)
    }
}

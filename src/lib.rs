use anyhow::{anyhow, Error};

use serde::{Deserialize, Serialize};

pub mod primitives;
pub mod wire;
pub mod cln_client;

use std::sync::{Arc, RwLock};

pub use crate::cln_client::{get_info, set_channel_fee, list_channels, onchain_balance};

#[derive(Default, Debug)]
pub struct Config {
    pub dynamic_fees: bool,
    pub dynamic_fee_min: i64,
    pub dynamic_fee_max: i64,
    pub dynamic_fee_interval: i64,
}

impl Config {
    pub fn default() -> Config {
        Config { 
            dynamic_fees: false, 
            dynamic_fee_min: 0, 
            dynamic_fee_max: 1000, 
            dynamic_fee_interval: 3600
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


pub async fn set_channel_fees(config: Arc<Config>) -> Result<(), Error> {
    log::debug!("Setting channel fees config: {:?}", config);
    let channels = list_channels().await.unwrap();
    for channel in channels {
        log::debug!("Channel under consideration: {:?}", channel);
        
        if channel.connected {
            let short_channel_id = &channel.short_channel_id.clone().expect("Short channel id not available");
            let fee_target = calculate_fee_target(&channel, &config).await.unwrap();
            let htlc_max_msat_target = calculate_htlc_max(&channel, &config).await.unwrap();
            log::debug!("Calculated target rate for channel (ChannelID: {:?}, Target: {:?})", &short_channel_id, &fee_target);
            let res = set_channel_fee(short_channel_id, fee_target, htlc_max_msat_target).await
                .map_err(|e| {
                    log::error!("Error setting a channel fee: {:?}", e);
                    e
                })?;
            log::info!("Channel set (ID: {:?} Fee: {}, Max HTLC: {})", &short_channel_id, fee_target, htlc_max_msat_target,);
            log::debug!("Result: {:?}", res);
        } else {
            log::info!("Skipping update as channel is not currently online");
        }
    }
    Ok(())
}

async fn calculate_htlc_max(channel: &wire::Channel, config: &Config) -> Result<u64, Error> {
    let ours: u64 = channel.our_amount_msat.msat() as u64;
    let values = [
        1_000, 
        100_000, 
        250_000, 
        1_000_000, 
        10_000_000, 
        50_000_000, 
        100_000_000, 
        250_000_000, 
        500_000_000, 
        1_000_000_000, 
        2_000_000_000, 
        3_000_000_000, 
        4_000_000_000, 
        5_000_000_000, 
        7_500_000_000, 
       10_000_000_000, 
       15_000_000_000, 
       20_000_000_000];
    let target = values.iter().rev().find(|&x| &ours >= x);
    let t = match target {
        Some(t) => t,
        None => &ours,
    };
    let capped = (0.9*(*t as f64)).round();
    Ok(capped as u64)
}

async fn calculate_fee_target(channel: &wire::Channel, config: &Config) -> Result<u32, Error> {
    let ours: f64 = channel.our_amount_msat.msat() as f64; 
    let total: f64 = channel.amount_msat.msat() as f64;
    let proportion = 1.0 - (ours / total);

    let min_threshold_ratio = 0.2;
    let max_threshold_ratio = 0.8;

    let max: f64 = config.dynamic_fee_max as f64;
    let min: f64 = config.dynamic_fee_min as f64;

    let range = max - min;
    log::debug!("Min {} Max {} Range {}", min, max, range);
    log::debug!("Target calculation (Ours: {}, Total: {}, Proportion: {}, Range: {}", ours, total, proportion, range);
    let target = if proportion <= min_threshold_ratio {
        min
    } else if proportion >= max_threshold_ratio {
        max
    } else { 
        let nom = proportion - min_threshold_ratio;
        let denom = max_threshold_ratio - min_threshold_ratio;
        ((nom / denom) * range) + min
    };
    
    Ok(((target / 10.0).floor() * 10.0) as u32)
}


#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_calculate_balanced_channel() {
        let config = Config {
            dynamic_fees: true,
            dynamic_fee_interval: 100,
            dynamic_fee_min: 100,
            dynamic_fee_max: 500,
        };

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

        let target = calculate_fee_target(&c, &config).await.unwrap();
        assert_eq!(target, 290)   
    }

    #[tokio::test]
    async fn calculate_htlc_max_channels() {
        let config = Config {
            dynamic_fees: true,
            dynamic_fee_interval: 100,
            dynamic_fee_min: 100,
            dynamic_fee_max: 500,
        };

        let test_cases = vec![
            (1_557_248_000, 900_000_000),
            (7_305_243_000, 4_500_000_000),
            (4_492_794_000, 3_600_000_000),
            (12_630_110_000, 9_000_000_000),
        ];

        for (ours, target) in test_cases {
            let c = wire::Channel {
                amount_msat: primitives::Amount {msat: 1000000000},
                our_amount_msat: primitives::Amount {msat: ours},
                connected: true,
                peer_id: "039b9e260863e6d8735325b286931d73be9f8e766970ad4fe1cbcc470cd8964635".to_string(),
                state: wire::ChannelState::CHANNELD_NORMAL,
                funding_txid: "724ee70bc1670368c3db3c2ebed30d00fa595774356cebf509196c68a471ca91".to_string(),
                funding_output: 0,
                short_channel_id: Some("123x123x0".to_string()),
            };
    
            let calc = calculate_htlc_max(&c, &config).await.unwrap();
            assert_eq!(calc, target)
        }
    }

    #[tokio::test]
    async fn calculate_imbalanced_channel() {
        let config = Config {
            dynamic_fees: true,
            dynamic_fee_interval: 100,
            dynamic_fee_min: 10,
            dynamic_fee_max: 500,
        };

        let test_cases = vec![
            (1000, 1000, 10),
            (1000, 0, 500),
            (1000, 200, 500),
            (1000, 205, 490),
            (1000, 795, 10)
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
    
            let target = calculate_fee_target(&c, &config).await.unwrap();
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

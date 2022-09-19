#[macro_use]
extern crate serde_json;
use serde::{Deserialize};
use cln_plugin::{options, Builder, Error, Plugin};

// Try RPC Connectivity
use anyhow::{anyhow, Result};
use tokio;
use std::time::Duration;
use tokio::{task, time}; 

use ceebalancer::{Config, get_info, onchain_balance, set_channel_fees};
use ceebalancer::primitives::Amount;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    if let Some(plugin) = Builder::new((), tokio::io::stdin(), tokio::io::stdout())
        .option(options::ConfigOption::new(
            "dynamic-fees",
            options::Value::Boolean(false),
            "Adjust fees dynamically to try to keep channels in balance",
        ))
        .option(options::ConfigOption::new(
            "dynamic-fee-min",
            options::Value::Integer(0),
            "Min fee for the dynamic range",
        ))
        .option(options::ConfigOption::new(
            "dynamic-fee-max",
            options::Value::Integer(1000),
            "Max fee for the dynamic range",
        ))
        
        .subscribe("forward_event", forward_handler)
        .subscribe("coin_movement", coin_movement_handler)
        .start()
        .await?
    {
        let config = load_configuration(&plugin).unwrap();

        test_get_info(&plugin).await.unwrap();

        let balance = onchain_balance().await.unwrap();
        log::debug!("Onchain Balance: {}", balance);

        if config.dynamic_fees {
            task::spawn(async move {
                loop {
                    time::sleep(Duration::from_secs(5)).await;
                    match set_channel_fees().await {
                        Ok(_) => {
                            log::debug!("Success");
                        },
                        Err(err) => {
                            log::warn!("Error in set channel fees.  Proceeding: {:?}", err);
                        },
                    };
                }
            });
        };
        

        plugin.join().await
        
    } else {
        Ok(())
    }
    // log::info!("I'm down here -not exactly sure why.");
    // Ok(())
}

fn load_configuration(plugin: &Plugin<()>) -> Result<Config, Error> {
    let c = Config::default();

    let dynamic_fees = match plugin.option("dynamic-fees") {
        Some(options::Value::Boolean(false)) => {
            log::debug!("`dynamic-fees` option is set to false.  Disabling");
            false
        }
        Some(options::Value::Boolean(true)) => {
            log::debug!("`dynamic-fees` option is set to true.  Enabling.");
            true
        }
        None => {
            log::info!("Missing 'dynamic-fees' option.  Disabling.");
            false
        }
        Some(o) => return Err(anyhow!("dynamic-fees is not a valid boolean: {:?}.", o)),
    };
    let dynamic_fee_min = match plugin.option("dynamic-fee-min") {
        Some(options::Value::Integer(i)) => i,
        None => {
            log::info!("Missing 'dynamic-fee-min' option.  Using default.");
            c.dynamic_fee_min
        }
        Some(o) => return Err(anyhow!("dynamic-fee-min is not a valid integer: {:?}.", o)),
    };
    let dynamic_fee_max = match plugin.option("dynamic-fee-max") {
        Some(options::Value::Integer(i)) => i,
        None => {
            log::info!("Missing 'dynamic-fee-max' option.  Using default.");
            c.dynamic_fee_min
        }
        Some(o) => return Err(anyhow!("dynamic-fee-max is not a valid integer: {:?}.", o)),
    };
    
    Config {
        dynamic_fees,
        dynamic_fee_min,
        dynamic_fee_max,
    }.make_current();
    log::info!("Configuration loaded: {:?}", Config::current());
    Ok(c)
}
#[macro_use]
extern crate serde_json;
use cln_plugin::{options, Builder, Error, Plugin};
use std::sync::Arc;
// Try RPC Connectivity
use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio;
use tokio::{task, time};

use ceebalancer::{get_info, onchain_balance, set_channel_fees, Config};

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
        .option(options::ConfigOption::new(
            "dynamic-fee-threshold",
            options::Value::Integer(20),
            "Channel threshold for payment adjustment (0-100)",
        ))
        .option(options::ConfigOption::new(
            "dynamic-fee-width",
            options::Value::Integer(100),
            "Fee step size",
        ))
        .option(options::ConfigOption::new(
            "dynamic-fee-update-interval",
            options::Value::Integer(7200),
            "Update/evaluation interval",
        ))
        .rpcmethod(
            "ceebalancer-adjust",
            "Manually triggers an adjustment run",
            adjust_handler,
        )
        .subscribe("forward_event", forward_handler)
        .start()
        .await?
    {
        let config = load_configuration(&plugin).unwrap();

        if config.dynamic_fees {
            task::spawn(async move {
                loop {
                    time::sleep(Duration::from_secs(
                        config.dynamic_fee_update_interval.try_into().unwrap(),
                    ))
                    .await;
                    log::info!("Initiating dynamic fee adjustment");
                    match set_channel_fees(config.clone()).await {
                        Ok(_) => {
                            log::debug!("Success");
                        }
                        Err(err) => {
                            log::warn!("Error in set channel fees.  Proceeding: {:?}", err);
                        }
                    };
                }
            });
        };

        plugin.join().await
    } else {
        Ok(())
    }
}

fn load_configuration(plugin: &Plugin<()>) -> Result<Arc<Config>, Error> {
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
    let dynamic_fee_threshold = match plugin.option("dynamic-fee-threshold") {
        Some(options::Value::Integer(i)) => 100.0 / i as f32,
        None => {
            log::info!("Missing 'dynamic-fee-threshold' option.  Using default.");
            c.dynamic_fee_threshold
        }
        Some(o) => {
            return Err(anyhow!(
                "dynamic-fee-threshold is not a valid integer: {:?}.",
                o
            ))
        }
    };
    let dynamic_fee_width = match plugin.option("dynamic-fee-width") {
        Some(options::Value::Integer(i)) => i,
        None => {
            log::info!("Missing 'dynamic-fee-width' option.  Using default.");
            c.dynamic_fee_width
        }
        Some(o) => {
            return Err(anyhow!(
                "dynamic-fee-width is not a valid integer: {:?}.",
                o
            ))
        }
    };
    let dynamic_fee_update_interval = match plugin.option("dynamic-fee-interval") {
        Some(options::Value::Integer(i)) => i,
        None => {
            log::info!("Missing 'dynamic-fee-interval' option.  Using default.");
            c.dynamic_fee_update_interval
        }
        Some(o) => {
            return Err(anyhow!(
                "dynamic-fee-interval is not a valid integer: {:?}.",
                o
            ))
        }
    };

    let dynamic_fee_threshold = 0.2;
    let dynamic_fee_width = 50;

    Config {
        dynamic_fees,
        dynamic_fee_min,
        dynamic_fee_max,
        dynamic_fee_threshold,
        dynamic_fee_width,
        dynamic_fee_update_interval,
    }
    .make_current();
    log::info!("Configuration loaded: {:?}", Config::current());
    Ok(Config::current())
}

async fn test_get_info(_plugin: &Plugin<()>) -> Result<(), Error> {
    log::debug!("Testing getinfo as a sanity check");
    let info = get_info().await.unwrap();
    log::info!("Got info: {}", info);
    Ok(())
}

async fn forward_handler(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::debug!("Got a forward notification: {}", v);
    Ok(())
}

async fn adjust_handler(p: Plugin<()>, _v: serde_json::Value) -> Result<serde_json::Value, Error> {
    let config = load_configuration(&p).unwrap();
    set_channel_fees(config.clone()).await.unwrap();
    Ok(json!("Success"))
}

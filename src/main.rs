//! This is a test plugin used to verify that we can compile and run
//! plugins using the Rust API against Core Lightning.

// https://lightning.readthedocs.io/PLUGINS.html

#[macro_use]
extern crate serde_json;
use cln_plugin::{options, Builder, Error, Plugin};
use anyhow::{anyhow, Context, Result};
use tokio;
use ceebalancer::{Config};



#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    if let Some(plugin) = Builder::new((), tokio::io::stdin(), tokio::io::stdout())
        // .option(options::ConfigOption::new(
        //     "broadcast-ad-capacity",
        //     options::Value::Boolean(false),
        //     "Publish threshold capacity for fulfilling liquidity ads",
        // ))
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
        
        .start()
        .await?
    {
        load_configuration(&plugin).unwrap();

        plugin.join().await
    } else {
        Ok(())
    }
}

fn load_configuration(plugin: &Plugin<()>) -> Result<(), Error> {
    let mut c = Config::default();

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

    dbg!(Config::current());
    Ok(())
}

async fn init_handler(_p: Plugin<()>, _v: serde_json::Value) -> Result<(), Error> {
    log::debug!("Received the init handler");
    Ok(())
}

async fn forward_handler(_p: Plugin<()>, v: serde_json::Value) -> Result<(), Error> {
    log::debug!("Got a forward notification: {}", v);
    Ok(())
}

async fn peer_connected_handler(
    _p: Plugin<()>,
    v: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    log::info!("Got a connect hook call: {}", v);
    Ok(json!({"result": "continue"}))
}
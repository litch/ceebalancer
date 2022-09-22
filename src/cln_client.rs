use cln_rpc::{
    model,
    ClnRpc, 
    Request,
};
use std::path::{Path};

use anyhow::{anyhow, Error};

use serde::{Deserialize, Serialize};

use std::sync::{Arc, RwLock};

use crate::{Config};
use crate::wire;
use crate::primitives;

pub async fn get_info() -> Result<String, Error> {
    let req = Request::Getinfo(model::GetinfoRequest {});
    
    Ok(call(req).await?)
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

pub async fn set_channel_fee(channel: wire::Channel, fee: u32, htlc_max_msat: u64) -> Result<(), Error> {
    let req = Request::SetChannel(model::SetchannelRequest {
        id: channel.short_channel_id.expect("Channel not ready yet"),
        feeppm: Some(fee),
        feebase: None,
        htlcmax: Some(cln_rpc::primitives::Amount::from_msat(htlc_max_msat)),
        htlcmin: None,
    });
    let res = call(req).await?;
    log::info!("Set channel: {:?}", res);

    Ok(())
}

async fn call(request: Request) -> Result<String, Error> {
    let path = Path::new("lightning-rpc");
    let mut rpc = ClnRpc::new(path).await?;
    let response = rpc
        .call(request.clone())
        .await
        .map_err(|e| anyhow!("Error calling {:?}: {:?}", request, e))?;
    
    Ok(serde_json::to_string_pretty(&response)?)
}

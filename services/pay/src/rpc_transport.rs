//! Read-only archival fallback for explicitly pruned log history.
use crate::native_chain::Error;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::{sync::Mutex, time::Instant};

// Shared by all scanners. One archive operation costs a chain check plus a log
// query; three seconds between starts stays below the reviewed public quota.
static ARCHIVE_NEXT: Mutex<Option<Instant>> = Mutex::const_new(None);

pub fn validate_endpoint(value: &str) -> Result<(), Error> {
    let url = reqwest::Url::parse(value)?;
    let local = url.host_str().is_some_and(|host| {
        host.trim_matches(['[', ']'])
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
    });
    if !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !(url.scheme() == "https" || url.scheme() == "http" && local)
    {
        return Err("archive RPC requires HTTPS or numeric loopback".into());
    }
    Ok(())
}

async fn post(
    client: &Client,
    endpoint: &str,
    method: &str,
    params: Value,
) -> Result<Value, Error> {
    Ok(client
        .post(endpoint)
        .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
        .send()
        .await
        .map_err(reqwest::Error::without_url)?
        .error_for_status()
        .map_err(reqwest::Error::without_url)?
        .json()
        .await
        .map_err(reqwest::Error::without_url)?)
}

fn result(value: Value) -> Result<Value, Error> {
    if value.get("error").is_some() {
        return Err("RPC returned an error".into());
    }
    value
        .get("result")
        .cloned()
        .ok_or_else(|| "missing RPC result".into())
}

fn bounded_logs(params: &Value) -> bool {
    let Some(array) = params.as_array() else {
        return false;
    };
    if array.len() != 1 {
        return false;
    }
    let filter = &array[0];
    let height = |name: &str| {
        filter[name]
            .as_str()
            .and_then(|s| s.strip_prefix("0x"))
            .and_then(|s| u64::from_str_radix(s, 16).ok())
    };
    match (height("fromBlock"), height("toBlock")) {
        (Some(start), Some(end)) => {
            filter.get("address").is_some() && end >= start && end - start < 50
        }
        _ => false,
    }
}

pub async fn read(
    client: &Client,
    primary: &str,
    archive: Option<&str>,
    chain_id: u64,
    method: &str,
    params: Value,
) -> Result<Value, Error> {
    let value = post(client, primary, method, params.clone()).await?;
    if method != "eth_getLogs" || value["error"]["code"].as_i64() != Some(-32701) {
        // Provider rate limits and generic outages are not reasons to switch
        // providers. Preserve fail-closed behavior for every other RPC error.
        return result(value);
    }
    let archive = archive.ok_or("Log history was pruned and no archive RPC is configured")?;
    validate_endpoint(archive)?;
    if !bounded_logs(&params) {
        return Err("archive logs require an explicit range of at most 50 blocks".into());
    }
    {
        let mut next = ARCHIVE_NEXT.lock().await;
        if let Some(deadline) = *next {
            tokio::time::sleep_until(deadline).await;
        }
        *next = Some(Instant::now() + Duration::from_secs(3));
    }
    let actual = result(post(client, archive, "eth_chainId", json!([])).await?)?;
    if crate::native_chain::number(&actual)? != chain_id {
        return Err("archive RPC chain mismatch".into());
    }
    result(post(client, archive, method, params).await?)
}

#[cfg(test)]
#[path = "rpc_transport_tests.rs"]
mod tests;

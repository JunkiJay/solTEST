use serde::Deserialize;
use serde_yaml::from_str;
use std::{fs, error::Error};
use reqwest::Client;
use futures::future::join_all;

#[derive(Debug, Deserialize)]
struct Config {
    wallets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: u64,
}

async fn get_balance(client: &Client, wallet: &str) -> Result<(String, u64), Box<dyn Error>> {
    let url = "https://api.mainnet-beta.solana.com";

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [wallet],
    });

    let res = client
        .post(url)
        .json(&request_body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(balance) = res.get("result").and_then(|r| r.get("value")).and_then(|v| v.as_u64()) {
        Ok((wallet.to_string(), balance))
    } else {
        Err(format!("Failed to parse balance for {}", wallet).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_str = fs::read_to_string("config.yaml")?;
    let config: Config = from_str(&config_str)?;

    let client = Client::new();

    let futures = config.wallets.iter()
        .map(|wallet| get_balance(&client, wallet));

    let results = join_all(futures).await;

    for result in results {
        match result {
            Ok((wallet, balance)) => println!("Wallet {}: {} lamports", wallet, balance),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

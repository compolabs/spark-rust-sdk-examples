use dotenv::dotenv;
use std::env;

use fuels::{accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::ContractId};
use std::str::FromStr;

use anyhow::Result;
use spark_market_sdk::SparkMarketContract;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id = env::var("BTC_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::connect(provider_url).await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id).unwrap();
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Getting Fees from Spark Market
    let matcher_fee = market.matcher_fee().await?.value;
    println!("matcher_fee: {:?}", matcher_fee);

    let protocol_fee = market.protocol_fee().await?.value;
    println!("protocol fee: {:?}", protocol_fee);

    Ok(())
}

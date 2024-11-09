use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider,
    accounts::wallet::WalletUnlocked,
    types::{AssetId, ContractId},
};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use spark_market_sdk::SparkMarketContract;
use spark_registry_sdk::SparkRegistryContract;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

fn parse_asset_id(asset_str: &str) -> Result<AssetId> {
    AssetId::from_str(asset_str).map_err(|e| anyhow!("Invalid Asset ID: {}", e))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Load environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let registry_contract_id = env::var("MARKET_REGISTRY")?;

    let wbtc_id = env::var("BTC_ID")?;
    let eth_id = env::var("ETH_ID")?;
    let usdc_id = env::var("USDC_ID")?;

    // Connect to the provider
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::connect(provider_url).await?;

    // Unlock wallet using the mnemonic
    let main_wallet = WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone()))?;

    // Initialize the SparkRegistry contract
    let contract_id = ContractId::from_str(&registry_contract_id)
        .map_err(|e| anyhow!("Invalid contract ID: {}", e))?;
    let registry = SparkRegistryContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Get configuration from Spark Registry
    let _config = registry.config().await?.value;

    // Parse asset IDs and construct asset pairs
    let btc_usdc = (parse_asset_id(&wbtc_id)?, parse_asset_id(&usdc_id)?);
    let eth_usdc = (parse_asset_id(&eth_id)?, parse_asset_id(&usdc_id)?);
    let assets = vec![eth_usdc];

    // Retrieve market ID from registry
    let registered_markets = registry.markets(assets.clone()).await?.value;

    // Extract and convert the market addresses, prepending "0x"
    let btc_usdc_contract_id = registered_markets
        .get(0)
        .and_then(|market| market.2)
        .map(|market_address| format!("0x{}", market_address))
        .ok_or_else(|| anyhow!("Failed to retrieve BTC/USDC market address"))?;

    let eth_usdc_contract_id = registered_markets
        .get(1)
        .and_then(|market| market.2)
        .map(|market_address| format!("0x{}", market_address))
        .ok_or_else(|| anyhow!("Failed to retrieve ETH/USDC market address"))?;

    // Initialize BTC and ETH market contracts
    let btc_contract_id = ContractId::from_str(&btc_usdc_contract_id)
        .map_err(|e| anyhow!("Failed to parse BTC/USDC contract ID: {}", e))?;
    let eth_contract_id = ContractId::from_str(&eth_usdc_contract_id)
        .map_err(|e| anyhow!("Failed to parse ETH/USDC contract ID: {}", e))?;

    let btc_market = SparkMarketContract::new(btc_contract_id.clone(), main_wallet.clone()).await;
    let eth_market = SparkMarketContract::new(eth_contract_id.clone(), main_wallet.clone()).await;

    println!("BTC/USDC market initialized: {:?}", btc_market.id());
    println!("ETH/USDC market initialized: {:?}", eth_market.id());

    Ok(())
}

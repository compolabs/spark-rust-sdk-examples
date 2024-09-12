use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::ContractId,
    types::Identity,
};
use std::str::FromStr;

use spark_market_sdk::SparkMarketContract;

use std::error::Error;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readble_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let market_contract_id = env::var("BTC_USDC_CONTRACT_ID")?;
    let _btc_id = env::var("BTC_ID")?;
    let _usdc_id = env::var("USDC_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();

    let market_contract_id = ContractId::from_str(&market_contract_id)?;
    let _market = SparkMarketContract::new(market_contract_id.clone(), main_wallet.clone()).await;

    // @dev we need to use src20 => deploy it to crates
    // let btc_id = ContractId::from_str(&btc_id);
    // let usdc_id = ContractId::from_str(&usdc_id);

    // Fuel wallet address
    let _wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    Ok(())
}

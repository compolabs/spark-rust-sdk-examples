use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, programs::calls::CallHandler,
    types::ContractId, types::Identity,
};
use std::str::FromStr;

use spark_market_sdk::MarketContract;
// use multiasset_sdk::MultiAssetContract;

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
    let btc_id = env::var("BTC_ID")?;
    let usdc_id = env::var("USDC_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();

    let market_contract_id = ContractId::from_str(&market_contract_id)?;
    let _market = MarketContract::new(market_contract_id.clone(), main_wallet.clone()).await;

    let btc_id = ContractId::from_str(&btc_id);
    let usdc_id = ContractId::from_str(&usdc_id);

    // let btc_src20 = Asset::new(main_wallet.clone(), btc_id.clone()).await;

    // Fuel wallet address
    let _wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    Ok(())
}

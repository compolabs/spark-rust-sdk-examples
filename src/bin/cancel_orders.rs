use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::AssetId,
    types::ContractId, types::Identity,
};
use std::str::FromStr;

use spark_market_sdk::{MarketContract, OrderType};
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
    let contract_id = env::var("BTC_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id)?;
    let market = MarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    // Fetching user orders
    let orders = market.user_orders(wallet_id).await?.value;
    println!("User Orders: {:?}", orders);

    // Cancel all open orders
    for order_id in orders {
        match market.cancel_order(order_id).await {
            Ok(_) => println!("Order {:?} canceled successfully", order_id),
            Err(e) => println!("Failed to cancel order {:?}: {:?}", order_id, e),
        }
    }

    Ok(())
}

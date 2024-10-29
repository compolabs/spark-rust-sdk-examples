use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked, ViewOnlyAccount},
    types::{AssetId, ContractId, Identity},
};
use std::str::FromStr;

use spark_market_sdk::{OrderType, SparkMarketContract};
use std::error::Error;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id = env::var("TRMP_KMLA_CONTRACT_ID")?;

    // Connect to provider
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::connect(provider_url).await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    let trmp_id: String = env::var("TRMP_ID")?;
    let kmla_id: String = env::var("KMLA_ID")?;

    // Getting asset balances
    let account = market.account(wallet_id).await.unwrap().value;
    println!("account: {:?}", account);

    // Depositing Assets
    let trmp_id = AssetId::from_str(&trmp_id)?;
    let trmp_amount = format_value_with_decimals(1, 9);

    let kmla_id: AssetId = AssetId::from_str(&kmla_id)?;
    let kmla_amount = format_value_with_decimals(1, 9);

    let trmp_bal = main_wallet.get_asset_balance(&trmp_id).await?;
    let kmla_bal = main_wallet.get_asset_balance(&kmla_id).await?;

    println!("trmp balance: {:?}", trmp_bal);
    println!("kmla balance: {:?}", kmla_bal);

    println!("Depositing TRMP");
    match market.deposit(trmp_amount, trmp_id).await {
        Ok(_) => {
            println!("Deposit Success");
            Ok(())
        }
        Err(e) => {
            print!("Deposit error: {:?}", e);
            Err(e)
        }
    }
    .unwrap();

    println!("Depositing KMLA");
    match market.deposit(kmla_amount, kmla_id).await {
        Ok(_) => {
            println!("Deposit Success");
            Ok(())
        }
        Err(e) => {
            print!("Deposit error: {:?}", e);
            Err(e)
        }
    }
    .unwrap();

    let account = market.account(wallet_id).await?.value;
    println!("account: {:?}", account);

    // Creating Buy / Sell Limit Orders

    // Buying 1
    let buy_amount = 1_000_000_000; // 1
    let order_type: OrderType = OrderType::Buy;
    let price: u64 = 1_000_000_000_u64;

    println!(
        "Opening Buy Order: {} TRMP at {} TRMP/KMLA",
        format_to_readable_value(buy_amount, 9),
        format_to_readable_value(price, 9)
    );
    match market.open_order(buy_amount, order_type, price).await {
        Ok(_) => {
            println!("Open Buy Order Success");
            Ok(())
        }
        Err(e) => {
            print!("Open Buy Order Error: {:?}", e);
            Err(e)
        }
    }
    .unwrap();

    // Selling 1
    let sell_amount = 1_000_000_000; // 1
    let order_type: OrderType = OrderType::Sell;
    let price: u64 = 1_000_000_000_u64;

    println!(
        "Opening Sell Order: {} TRMP at {} TRMP/KMLA",
        format_to_readable_value(sell_amount, 9),
        format_to_readable_value(price, 9)
    );
    match market.open_order(sell_amount, order_type, price).await {
        Ok(_) => {
            println!("Open Sell Order Success");
            Ok(())
        }
        Err(e) => {
            print!("Open Sell Order Error: {:?}", e);
            Err(e)
        }
    }
    .unwrap();

    let orders = market.user_orders(wallet_id).await?.value;
    println!("orders {:?}", orders.len());

    let account = market.account(wallet_id).await.unwrap().value;
    println!("account: {:?}", account);

    Ok(())
}

// match_order_trmp_kmla.rs

use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked, ViewOnlyAccount},
    types::{AssetId, ContractId, Identity},
};
use std::error::Error;
use std::str::FromStr;

use anyhow::Result;
use spark_market_sdk::{OrderType, SparkMarketContract};

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
    println!("Wallet Address: {:?}", main_wallet.address().to_string());

    // Asset IDs for TRMP and KMLA
    let trmp_id: String = env::var("TRMP_ID")?;
    let kmla_id: String = env::var("KMLA_ID")?;

    // Getting asset balances
    let account = market.account(wallet_id).await?.value;
    println!("Account Details: {:?}", account);

    // Parse Asset IDs
    let trmp_id = AssetId::from_str(&trmp_id)?;
    let kmla_id = AssetId::from_str(&kmla_id)?;

    // Define deposit amounts (using meme_test_tokens amounts)
    let trmp_amount = format_value_with_decimals(1, 9); // 1 TRMP with 9 decimals
    let kmla_amount = format_value_with_decimals(1, 9); // 1 KMLA with 9 decimals

    // Check current balances
    let trmp_bal = main_wallet.get_asset_balance(&trmp_id).await?;
    let kmla_bal = main_wallet.get_asset_balance(&kmla_id).await?;

    println!("TRMP Balance: {:?}", trmp_bal);
    println!("KMLA Balance: {:?}", kmla_bal);

    // Depositing TRMP
    println!("Depositing TRMP...");
    match market.deposit(trmp_amount, trmp_id).await {
        Ok(_) => println!("TRMP Deposit Success"),
        Err(e) => {
            eprintln!("TRMP Deposit Error: {:?}", e);
            return Err(e.into());
        }
    }

    // Depositing KMLA
    println!("Depositing KMLA...");
    match market.deposit(kmla_amount, kmla_id).await {
        Ok(_) => println!("KMLA Deposit Success"),
        Err(e) => {
            eprintln!("KMLA Deposit Error: {:?}", e);
            return Err(e.into());
        }
    }

    // Refresh account details after deposit
    let account = market.account(wallet_id).await?.value;
    println!("Updated Account Details: {:?}", account);

    // Creating Buy / Sell Limit Orders

    // Example: Buying 1 TRMP at a specified price
    let buy_amount = format_value_with_decimals(1, 9); // 1 TRMP
    let buy_order_type: OrderType = OrderType::Buy;
    let buy_price = format_value_with_decimals(1, 9);

    println!(
        "Opening Buy Order: {} TRMP at {} KMLA/TRMP",
        format_to_readable_value(buy_amount, 9),
        format_to_readable_value(buy_price, 9)
    );
    let buy_order_id = match market
        .open_order(buy_amount, buy_order_type, buy_price)
        .await
    {
        Ok(order_id) => {
            println!("Buy Order Opened Successfully: {:?}", order_id.value);
            order_id.value
        }
        Err(e) => {
            eprintln!("Buy Order Error: {:?}", e);
            return Err(e.into());
        }
    };

    // Example: Selling 1 TRMP at a specified price
    let sell_amount = format_value_with_decimals(1, 9); // 1 TRMP
    let sell_order_type: OrderType = OrderType::Sell;
    let sell_price = format_value_with_decimals(1, 9);

    println!(
        "Opening Sell Order: {} TRMP at {} KMLA/TRMP",
        format_to_readable_value(sell_amount, 9),
        format_to_readable_value(sell_price, 9)
    );
    let sell_order_id = match market
        .open_order(sell_amount, sell_order_type, sell_price)
        .await
    {
        Ok(order_id) => {
            println!("Sell Order Opened Successfully: {:?}", order_id.value);
            order_id.value
        }
        Err(e) => {
            eprintln!("Sell Order Error: {:?}", e);
            return Err(e.into());
        }
    };

    // Matching the two orders
    println!(
        "Matching Orders: {:?} and {:?}",
        buy_order_id, sell_order_id
    );
    match market.match_order_pair(buy_order_id, sell_order_id).await {
        Ok(_) => println!("Orders Matched Successfully"),
        Err(e) => {
            eprintln!("Order Matching Error: {:?}", e);
            return Err(e.into());
        }
    }

    // Final account details
    let final_account = market.account(wallet_id).await?.value;
    println!("Final Account Details: {:?}", final_account);

    Ok(())
}

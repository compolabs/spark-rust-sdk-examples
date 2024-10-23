use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::AssetId,
    types::ContractId, types::Identity,
};
use std::str::FromStr;

use anyhow::Result;
use spark_market_sdk::{OrderType, SparkMarketContract};

pub fn format_value_with_decimals(value: f64, decimals: u32) -> u64 {
    (value * 10f64.powi(decimals as i32)) as u64
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10f64.powi(decimals as i32)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id = env::var("BTC_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id).unwrap();
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("Wallet Address: {:?}", main_wallet.address().to_string());

    let btc_id: String = env::var("BTC_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    // Depositing Assets
    let btc_id = AssetId::from_str(&btc_id).unwrap();
    let btc_amount = format_value_with_decimals(1.0, 8); // Deposit 1 BTC

    let usdc_id = AssetId::from_str(&usdc_id).unwrap();
    let usdc_amount = format_value_with_decimals(70_000.0, 6); // Deposit 70,000 USDC

    println!("Depositing BTC...");
    market.deposit(btc_amount, btc_id).await?;
    println!("BTC Deposit Successful");

    println!("Depositing USDC...");
    market.deposit(usdc_amount, usdc_id).await?;
    println!("USDC Deposit Successful");

    // Number of orders to create
    let n_orders = 10;

    // Buy/Sell order amounts and price
    let buy_amount = format_value_with_decimals(0.1, 8); // 0.1 BTC per order
    let sell_amount = format_value_with_decimals(0.1, 8); // 0.1 BTC per order
    let price = format_value_with_decimals(70_000.0, 9); // 70,000 USDC per BTC with 9 decimals

    // Vectors to store order IDs
    let mut buy_order_ids = Vec::new();
    let mut sell_order_ids = Vec::new();

    // Creating Buy Orders
    for _ in 0..n_orders {
        println!(
            "Opening Buy Order: {} BTC at ${} per BTC",
            format_to_readable_value(buy_amount, 8),
            format_to_readable_value(price, 9)
        );
        let order_id = market
            .open_order(buy_amount, OrderType::Buy, price)
            .await?
            .value;
        buy_order_ids.push(order_id);
    }

    // Creating Sell Orders
    for _ in 0..n_orders {
        println!(
            "Opening Sell Order: {} BTC at ${} per BTC",
            format_to_readable_value(sell_amount, 8),
            format_to_readable_value(price, 9)
        );
        let order_id = market
            .open_order(sell_amount, OrderType::Sell, price)
            .await?
            .value;
        sell_order_ids.push(order_id);
    }

    // Matching Orders
    for i in 0..n_orders {
        let buy_order_id = buy_order_ids[i];
        let sell_order_id = sell_order_ids[i];

        println!(
            "Matching Orders: Buy {:?} with Sell {:?}",
            buy_order_id, sell_order_id
        );
        market.match_order_pair(buy_order_id, sell_order_id).await?;
        println!("Orders Matched Successfully");
    }

    // Fetch and display the account balance after matching
    let account = market.account(wallet_id).await?.value;
    println!("Account after matching orders: {:?}", account);

    Ok(())
}

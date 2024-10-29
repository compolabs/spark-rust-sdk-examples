use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::AssetId,
    types::ContractId, types::Identity,
};
use std::str::FromStr;

use anyhow::Result;
use spark_market_sdk::{OrderType, SparkMarketContract};
use tokio::time::{sleep, Duration};

// Add these imports
use reqwest;
use serde_json::Value;

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
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::connect(provider_url).await?;

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
    let usdc_amount = format_value_with_decimals(80_000.0, 6); // Deposit 80,000 USDC

    println!("Depositing BTC...");
    market.deposit(btc_amount, btc_id).await?;
    println!("BTC Deposit Successful");

    println!("Depositing USDC...");
    market.deposit(usdc_amount, usdc_id).await?;
    println!("USDC Deposit Successful");

    // Fetch the current price of Bitcoin from CoinGecko API
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
    let response = reqwest::get(url).await?.json::<Value>().await?;
    let current_price = response["bitcoin"]["usd"]
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse BTC price from API response"))?;
    println!("Current BTC price: ${:.2}", current_price);

    let base_price = current_price;

    // Buy/Sell order amounts
    let buy_amount = format_value_with_decimals(0.1, 8); // 0.1 BTC per order
    let sell_amount = format_value_with_decimals(0.1, 8); // 0.1 BTC per order

    // Vectors to store order IDs
    let mut buy_order_ids = Vec::new();
    let mut sell_order_ids = Vec::new();

    // Number of orders to create
    let n_orders = 10;

    // Creating Buy Orders
    for i in 0..n_orders {
        // Adjust the price
        let price_adjustment = if i % 2 == 0 {
            ((i / 2 + 1) as f64) * 1000.0 // Increase by 1000
        } else {
            -((i / 2 + 1) as f64) * 1000.0 // Decrease by 1000
        };
        let price = base_price + price_adjustment;

        let price_formatted = format_value_with_decimals(price, 9); // Price with 9 decimals

        println!(
            "Opening Buy Order: {} BTC at ${:.2} per BTC",
            format_to_readable_value(buy_amount, 8),
            price
        );
        let order_id = market
            .open_order(buy_amount, OrderType::Buy, price_formatted)
            .await?
            .value;
        buy_order_ids.push(order_id);
    }

    // Creating Sell Orders
    for i in 0..n_orders {
        // Adjust the price (same as for buy orders)
        let price_adjustment = if i % 2 == 0 {
            ((i / 2 + 1) as f64) * 1000.0 // Increase by 1000
        } else {
            -((i / 2 + 1) as f64) * 1000.0 // Decrease by 1000
        };
        let price = base_price + price_adjustment;

        let price_formatted = format_value_with_decimals(price, 9); // Price with 9 decimals

        println!(
            "Opening Sell Order: {} BTC at ${:.2} per BTC",
            format_to_readable_value(sell_amount, 8),
            price
        );
        let order_id = market
            .open_order(sell_amount, OrderType::Sell, price_formatted)
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

        // Wait 3 seconds before matching the next pair
        sleep(Duration::from_secs(3)).await;
    }

    // Fetch and display the account balance after matching
    let account = market.account(wallet_id).await?.value;
    println!("Account after matching orders: {:?}", account);

    Ok(())
}

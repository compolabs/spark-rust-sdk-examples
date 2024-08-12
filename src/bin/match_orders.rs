use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::AssetId,
    types::ContractId, types::Identity,
};
use std::str::FromStr;

use anyhow::Result;
use spark_market_sdk::{MarketContract, OrderType};

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
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id).unwrap();
    let market = MarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    let btc_id: String = env::var("BTC_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    // Getting asset balances
    if let Some(account) = market.account(wallet_id).await?.value {
        let liquid_base = account.liquid.base;
        let liquid_quote = account.liquid.quote;

        println!("BTC Balance: {:?}", liquid_base);
        println!("USDC Balance: {:?}", liquid_quote);
    } else {
        println!("Account not found or has no balance.");
    }

    // Depositing Assets
    let btc_id = AssetId::from_str(&btc_id).unwrap();
    let btc_amount = format_value_with_decimals(1, 7);

    let usdc_id = AssetId::from_str(&usdc_id).unwrap();
    let usdc_amount = format_value_with_decimals(10_000, 5);

    println!("Depositing BTC");
    market.deposit(btc_amount, btc_id).await?;
    println!("Deposit Success");

    println!("Depositing USDC");
    market.deposit(usdc_amount, usdc_id).await?;
    println!("Deposit Success");

    // Creating Buy / Sell Limit Orders

    // Buying 10_000 USDC worth of BTC
    let buy_amount: u64 = usdc_amount;
    let order_type: OrderType = OrderType::Buy;
    let price: u64 = 70_000_000_000_000_u64;

    println!(
        "Opening Buy Order: {} USDC at {} BTC/USDC",
        format_to_readable_value(buy_amount, 6),
        format_to_readable_value(price, 9)
    );
    let order_id0 = market
        .open_order(buy_amount, order_type, price)
        .await?
        .value;

    // Selling 0.1 BTC for 70k USDC
    let sell_amount: u64 = btc_amount;
    let order_type = OrderType::Sell;
    let price = 70_000_000_000_000_u64;

    println!(
        "Opening Sell Order: {} BTC at {} BTC/USDC",
        format_to_readable_value(sell_amount, 8),
        format_to_readable_value(price, 9)
    );
    let order_id1 = market
        .open_order(sell_amount, order_type, price)
        .await?
        .value;

    // Matching the two orders
    println!("Matching Orders: {:?} and {:?}", order_id0, order_id1);
    market.match_order_pair(order_id0, order_id1).await?;

    println!("Orders Matched Successfully");

    Ok(())
}

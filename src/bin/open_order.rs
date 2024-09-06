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
    let contract_id = env::var("BTC_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    let btc_id: String = env::var("BTC_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    // Getting asset balances
    let account = market.account(wallet_id).await.unwrap().value;
    println!("account: {:?}", account);

    // Depositing Assets
    let btc_id = AssetId::from_str(&btc_id)?;
    let btc_amount = format_value_with_decimals(1, 8);

    let usdc_id = AssetId::from_str(&usdc_id)?;
    let usdc_amount = format_value_with_decimals(10_000, 6);

    let btc_bal = main_wallet.get_asset_balance(&btc_id).await?;
    let usd_bal = main_wallet.get_asset_balance(&usdc_id).await?;

    println!("btc balance: {:?}", btc_bal);
    println!("usdc balance: {:?}", usd_bal);

    println!("Depositing BTC");
    match market.deposit(btc_amount, btc_id).await {
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

    println!("Depositing USDC");
    match market.deposit(usdc_amount, usdc_id).await {
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

    // Buying 0.1 BTC
    let buy_amount = 10_000_000; // 0.1 BTC
    let order_type: OrderType = OrderType::Buy;
    let price: u64 = 70_000_000_000_000_u64;

    println!(
        "Opening Buy Order: {} BTC at {} BTC/USDC",
        format_to_readable_value(buy_amount, 8),
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

    // Selling 0.1 BTC for 70k USDC
    let sell_amount: u64 = buy_amount;
    let order_type = OrderType::Sell;
    let price = 70_000_000_000_000_u64;

    println!(
        "Opening Sell Order: {} BTC at {} BTC/USDC",
        format_to_readable_value(sell_amount, 8),
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

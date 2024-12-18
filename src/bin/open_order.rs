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
    let contract_id = env::var("ETH_USDC_CONTRACT_ID")?;

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

    let eth_id: String = env::var("ETH_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    // Getting asset balances
    let account = market.account(wallet_id).await.unwrap().value;
    println!("account: {:?}", account);

    // Depositing Assets
    let eth_id = AssetId::from_str(&eth_id)?;
    let eth_amount = format_value_with_decimals(1, 6);

    let usdc_id = AssetId::from_str(&usdc_id)?;
    let usdc_amount = format_value_with_decimals(5, 6);

    let eth_bal = main_wallet.get_asset_balance(&eth_id).await?;
    let usd_bal = main_wallet.get_asset_balance(&usdc_id).await?;

    println!("eth balance: {:?}", eth_bal);
    println!("usdc balance: {:?}", usd_bal);

    println!("Depositing eth");
    match market.deposit(eth_amount, eth_id).await {
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

    // Buying 0.001 ETH
    let buy_amount = 1000000;
    let order_type: OrderType = OrderType::Buy;
    let price: u64 = 3_201_000_000_000_u64;

    println!(
        "Opening Buy Order: {} ETH at {} ETH/USDC",
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

    // Selling 0.001 ETH for 3002 USDC
    let sell_amount: u64 = 1000000;
    let order_type = OrderType::Sell;
    let price = 3_402_000_000_000_u64;

    println!(
        "Opening Sell Order: {} ETH at {} ETH/USDC",
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
    println!("orders {:?}", orders);

    let account = market.account(wallet_id).await.unwrap().value;
    println!("account: {:?}", account);

    Ok(())
}

use dotenv::dotenv;
use rand::Rng;
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

    let btc_id: String = env::var("BTC_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    // Getting asset balances
    let account = market.account(wallet_id).await.unwrap().value;
    println!("account balance: {:?}", account);

    // Depositing Assets
    let btc_id = AssetId::from_str(&btc_id)?;
    let btc_amount = format_value_with_decimals(1, 9);

    let usdc_id = AssetId::from_str(&usdc_id)?;
    let usdc_amount = format_value_with_decimals(100_000, 6);

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

    // Creating Buy / Sell Limit Orders
    let mut rng = rand::thread_rng();

    for price in (46000..=72000).step_by(1000) {
        let price = format_value_with_decimals(price, 9);

        // Randomize the order amounts between 2 and 10 USD
        let buy_amount = format_value_with_decimals(rng.gen_range(1..50), 6);
        let sell_amount = format_value_with_decimals(rng.gen_range(3..30), 6);

        // Buy Order
        let order_type: OrderType = OrderType::Buy;
        println!(
            "Opening Buy Order: {} BTC at {} BTC/USDC",
            format_to_readble_value(buy_amount, 8),
            format_to_readble_value(price, 9)
        );
        match market.open_order(buy_amount, order_type, price).await {
            Ok(_) => println!("Open Buy Order Success"),
            Err(e) => println!("Open Buy Order Error: {:?}", e),
        }

        // Sell Order
        let order_type: OrderType = OrderType::Sell;
        println!(
            "Opening Sell Order: {} BTC at {} BTC/USDC",
            format_to_readble_value(sell_amount, 8),
            format_to_readble_value(price, 9)
        );
        match market.open_order(sell_amount, order_type, price).await {
            Ok(_) => println!("Open Sell Order Success"),
            Err(e) => println!("Open Sell Order Error: {:?}", e),
        }
    }

    let orders = market.user_orders(wallet_id).await?.value;
    println!("orders {:?}", orders);

    Ok(())
}

use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, prelude::CallHandler,
    types::AssetId, types::ContractId, types::Identity,
};
use std::str::FromStr;

use spark_market_sdk::{AssetType, MarketContract, OrderType};
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

    // Getting asset balances
    if let Some(account) = market.account(wallet_id).await.unwrap().value {
        let liquid_base = account.liquid.base;
        let liquid_quote = account.liquid.quote;

        println!("BTC Balance: {:?}", liquid_base);
        println!("USDC Balance: {:?}", liquid_quote);
    } else {
        println!("Account not found or has no balance.");
    }

    // Asset Amounts
    let btc_amount = format_value_with_decimals(1, 6);
    let usdc_amount = format_value_with_decimals(10_000, 5);

    // Creating Buy / Sell Limit Orders

    // Buying 10_000 USDC worth of BTC
    let buy_amount: u64 = usdc_amount;
    let order_type: OrderType = OrderType::Buy;
    let price: u64 = 70_000_000_000_000_u64;

    println!(
        "Opening Buy Order: {} USDC at {} BTC/USDC",
        format_to_readble_value(buy_amount, 4),
        format_to_readble_value(price, 9)
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
    let sell_amount: u64 = btc_amount;
    let order_type = OrderType::Sell;
    let price = 70_000_000_000_000_u64;

    println!(
        "Opening Sell Order: {} BTC at {} BTC/USDC",
        format_to_readble_value(sell_amount, 7),
        format_to_readble_value(price, 9)
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

    // Fetching user orders
    let orders = market.user_orders(wallet_id).await?.value;
    println!("User Orders: {:?}", orders);

    // Canceling Orders
    let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());

    for order_id in orders.clone() {
        let cancel_order_call = market.get_instance().methods().cancel_order(order_id);
        multi_call_handler = multi_call_handler.add_call(cancel_order_call);
    }

    // Execute all the prepared calls in a single transaction
    println!("Cancelling {} orders", orders.len());
    if orders.len() > 0 {
        let cancel_order_multicall_tx = multi_call_handler.submit().await?;
        println!(
            "Submitted cancel transaction: {:?}",
            cancel_order_multicall_tx
        );
    } else {
        println!("No orders to cancel");
    }

    // Withdraw assets after canceling orders
    let btc_withdraw_amount = btc_amount;
    let usdc_withdraw_amount = usdc_amount;

    println!("Withdrawing BTC");
    match market.withdraw(btc_withdraw_amount, AssetType::Base).await {
        Ok(_) => {
            println!("Withdraw BTC Success");
            Ok(())
        }
        Err(e) => {
            print!("Withdraw BTC Error: {:?}", e);
            Err(e)
        }
    }
    .unwrap();

    println!("Withdrawing USDC");
    match market
        .withdraw(usdc_withdraw_amount, AssetType::Quote)
        .await
    {
        Ok(_) => {
            println!("Withdraw USDC Success");
            Ok(())
        }
        Err(e) => {
            print!("Withdraw USDC Error: {:?}", e);
            Err(e)
        }
    }
    .unwrap();

    Ok(())
}

use dotenv::dotenv;
use std::{env, error::Error, str::FromStr};

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    programs::calls::CallHandler,
    types::{AssetId, ContractId, Identity},
};

use spark_market_sdk::{OrderType, SparkMarketContract};

use tokio::time::{sleep, Duration};

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

// Might fail if you don't have enough testnet ETH
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

    // Depositing Assets
    let eth_id = AssetId::from_str(&eth_id)?;
    let eth_amount = format_value_with_decimals(1, 9);

    let usdc_id = AssetId::from_str(&usdc_id)?;
    let usdc_amount = format_value_with_decimals(3_000, 6);

    let mut multi_call_handler: CallHandler<
        WalletUnlocked,
        Vec<fuels::programs::calls::ContractCall>,
        (),
    > = CallHandler::new_multi_call(main_wallet.clone());

    // Deposit Calls
    let deposit_btc_call = market.deposit_call_handler(eth_amount, eth_id);
    let deposit_usdc_call = market.deposit_call_handler(usdc_amount, usdc_id);

    multi_call_handler = multi_call_handler.add_call(deposit_btc_call);
    multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);

    // Creating Buy / Sell Limit Orders in a single transaction

    let protocol_fee = market.protocol_fee().await?.value;
    println!("protocol_fee: {:?}", protocol_fee);

    // let matcher_fee = market.matcher_fee().await?.value as u64;
    let buy_order_type = OrderType::Buy;
    let buy_order_amount = 500_000; // 0.005 ETH
    let buy_start_price = 2_000u64;
    let sell_order_amount = 1_000_000;
    let sell_start_price = 2_100u64;
    let step = 500;

    for i in 0..5 {
        let buy_open_price = (buy_start_price + i * step) * 1_000_000_000_u64;
        let sell_open_price = (sell_start_price + i * step) * 1_000_000_000_u64;

        let buy_open_order_call = market.open_order_call_handler(
            buy_order_amount,
            buy_order_type.clone(),
            buy_open_price,
        );

        let sell_open_order_call =
            market.open_order_call_handler(sell_order_amount, OrderType::Sell, sell_open_price);

        multi_call_handler = multi_call_handler.add_call(buy_open_order_call);
        multi_call_handler = multi_call_handler.add_call(sell_open_order_call);
    }

    // Execute all the prepared calls in a single transaction (deposit & open orders)
    let _multicall_tx_result = multi_call_handler.submit().await?;
    // println!("Multicall tx result: {:?}", multicall_tx_result);

    sleep(Duration::from_secs(2)).await;

    let orders = market.user_orders(wallet_id).await?.value;
    println!("Number of Orders: {:?}", orders.len());

    Ok(())
}

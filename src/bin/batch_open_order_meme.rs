// batch_open_order_trmp_kmla.rs

use dotenv::dotenv;
use std::{env, error::Error, str::FromStr};

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    prelude::CallParameters,
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

    let orders = market.user_orders(wallet_id).await?.value;
    println!("Initial Number of Orders: {:?}", orders.len());

    // Asset IDs for TRMP and KMLA
    let trmp_id: String = env::var("TRMP_ID")?;
    let kmla_id: String = env::var("KMLA_ID")?;

    // Depositing Assets
    let trmp_id = AssetId::from_str(&trmp_id)?;
    let trmp_amount = format_value_with_decimals(1000, 9); // 1 TRMP with 9 decimals

    let kmla_id = AssetId::from_str(&kmla_id)?;
    let kmla_amount = format_value_with_decimals(1000, 9); // 1 KMLA with 9 decimals

    let account_before = market.account(wallet_id.clone()).await?.value;
    println!(
        "Market account before deposit and order creation: {:?}",
        account_before
    );

    let mut multi_call_handler: CallHandler<
        WalletUnlocked,
        Vec<fuels::programs::calls::ContractCall>,
        (),
    > = CallHandler::new_multi_call(main_wallet.clone());

    // Deposit Calls
    let deposit_trmp_call_params = CallParameters::new(trmp_amount, trmp_id, 20_000_000);
    let deposit_trmp_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_trmp_call_params)
        .unwrap();

    let deposit_kmla_call_params = CallParameters::new(kmla_amount, kmla_id, 20_000_000);
    let deposit_kmla_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_kmla_call_params)
        .unwrap();

    multi_call_handler = multi_call_handler.add_call(deposit_trmp_call);
    multi_call_handler = multi_call_handler.add_call(deposit_kmla_call);

    let protocol_fee = market.protocol_fee().await?.value;
    println!("Protocol Fee: {:?}", protocol_fee);

    // Define order parameters
    let open_order_call_params = CallParameters::default();

    let buy_order_type = OrderType::Buy;
    let buy_order_amount = format_value_with_decimals(1, 8); // 100 TRMP
    let buy_start_price = format_value_with_decimals(570, 8); // Example: 50 KMLA per TRMP

    let sell_order_type = OrderType::Sell;
    let sell_order_amount = format_value_with_decimals(1, 8); // 100 TRMP
    let sell_start_price = format_value_with_decimals(571, 8); // Example: 51 KMLA per TRMP

    let step = 500_000_000; // Adjusted for 9 decimals (e.g., 0.5 KMLA)

    // Creating Buy / Sell Limit Orders in a single transaction
    for i in 0..20 {
        let buy_open_price = buy_start_price + i * step;
        let sell_open_price = sell_start_price + i * step;

        let buy_open_order_call = market
            .get_instance()
            .methods()
            .open_order(buy_order_amount, buy_order_type.clone(), buy_open_price)
            .call_params(open_order_call_params.clone())
            .unwrap();

        let sell_open_order_call = market
            .get_instance()
            .methods()
            .open_order(sell_order_amount, sell_order_type.clone(), sell_open_price)
            .call_params(open_order_call_params.clone())
            .unwrap();

        multi_call_handler = multi_call_handler.add_call(buy_open_order_call);
        multi_call_handler = multi_call_handler.add_call(sell_open_order_call);
    }

    // Execute all the prepared calls in a single transaction (deposit & open orders)
    let multicall_tx_result = multi_call_handler.submit().await?;

    let tx_id = multicall_tx_result.tx_id();
    println!("Multicall Transaction ID: 0x{:?}", tx_id);

    // Wait for the transaction to be processed
    sleep(Duration::from_secs(5)).await; // Increased wait time to ensure transaction completion

    let orders = market.user_orders(wallet_id).await?.value;
    println!("Number of Orders: {:?}", orders.len());

    let account_after = market.account(wallet_id.clone()).await?.value;
    println!(
        "Market account after deposit and order creation: {:?}",
        account_after
    );

    Ok(())
}

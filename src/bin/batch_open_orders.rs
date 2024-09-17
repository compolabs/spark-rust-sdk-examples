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

    // Depositing Assets
    let btc_id = AssetId::from_str(&btc_id)?;
    let btc_amount = format_value_with_decimals(1, 6);

    let usdc_id = AssetId::from_str(&usdc_id)?;
    let usdc_amount = format_value_with_decimals(3_000, 6);

    let account = market.account(wallet_id.clone()).await?.value;
    println!(
        "market account before deposit and order creation: {:?}",
        account
    );

    let mut multi_call_handler: CallHandler<
        WalletUnlocked,
        Vec<fuels::programs::calls::ContractCall>,
        (),
    > = CallHandler::new_multi_call(main_wallet.clone());

    // Deposit Calls
    let deposit_btc_call_params = CallParameters::new(btc_amount, btc_id, 20_000_000);
    let deposit_btc_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_btc_call_params)
        .unwrap();

    let deposit_usdc_call_params = CallParameters::new(usdc_amount, usdc_id, 20_000_000);
    let deposit_usdc_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_usdc_call_params)
        .unwrap();

    multi_call_handler = multi_call_handler.add_call(deposit_btc_call);
    multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);

    let protocol_fee = market.protocol_fee().await?.value;
    println!("protocol_fee: {:?}", protocol_fee);

    // let matcher_fee = market.matcher_fee().await?.value as u64;
    let open_order_call_params = CallParameters::default();

    let buy_order_type = OrderType::Buy;
    let buy_order_amount = 100_000; // 0.001 BTC
    let buy_start_price = 50_000u64; // 50k
    let sell_order_amount = 100_000; // 0.001 BTC
    let sell_start_price = 51_000u64; // 51k
    let step = 500;

    // Creating Buy / Sell Limit Orders in a single transaction
    for i in 0..5 {
        let buy_open_price = (buy_start_price + i * step) * 1_000_000_000_u64;
        let sell_open_price = (sell_start_price + i * step) * 1_000_000_000_u64;

        let buy_open_order_call = market
            .get_instance()
            .methods()
            .open_order(buy_order_amount, buy_order_type.clone(), buy_open_price)
            .call_params(open_order_call_params.clone())
            .unwrap();

        let sell_open_order_call = market
            .get_instance()
            .methods()
            .open_order(sell_order_amount, OrderType::Sell, sell_open_price)
            .call_params(open_order_call_params.clone())
            .unwrap();

        multi_call_handler = multi_call_handler.add_call(buy_open_order_call);
        multi_call_handler = multi_call_handler.add_call(sell_open_order_call);
    }

    // Execute all the prepared calls in a single transaction (deposit & open orders)
    let multicall_tx_result = multi_call_handler.submit().await?;

    let tx_id = multicall_tx_result.tx_id();
    println!("multicall transaction id: 0x{:?}", tx_id);

    sleep(Duration::from_secs(1)).await;

    let orders = market.user_orders(wallet_id).await?.value;
    println!("Number of Orders: {:?}", orders.len());

    let account = market.account(wallet_id.clone()).await?.value;
    println!(
        "market account after deposit and order creation: {:?}",
        account
    );

    Ok(())
}

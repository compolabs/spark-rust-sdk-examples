use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, prelude::CallParameters,
    programs::calls::CallHandler, types::AssetId, types::ContractId, types::Identity,
};
use std::str::FromStr;

use spark_market_sdk::{OrderType, SparkMarketContract};
use std::error::Error;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Load environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id = env::var("BTC_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Get wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("Wallet address: {:?}", main_wallet.address().to_string());

    let btc_id: String = env::var("BTC_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    // Deposit assets (simplified for BTC and USDC)
    let btc_id = AssetId::from_str(&btc_id)?;
    let usdc_id = AssetId::from_str(&usdc_id)?;

    let btc_amount = format_value_with_decimals(1, 8); // 1 BTC
    let usdc_amount = format_value_with_decimals(10_000, 6); // 10,000 USDC

    let mut multi_call_handler: CallHandler<
        WalletUnlocked,
        Vec<fuels::programs::calls::ContractCall>,
        (),
    > = CallHandler::new_multi_call(main_wallet.clone());

    // Deposit BTC
    let deposit_btc_call_params = CallParameters::new(btc_amount, btc_id, 5_000_000);
    let deposit_btc_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_btc_call_params)
        .unwrap();
    multi_call_handler = multi_call_handler.add_call(deposit_btc_call);

    // Deposit USDC
    let deposit_usdc_call_params = CallParameters::new(usdc_amount, usdc_id, 5_000_000);
    let deposit_usdc_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_usdc_call_params)
        .unwrap();
    multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);

    // Add Buy Order (simplified)
    let buy_order_type = OrderType::Buy;
    let buy_order_amount = 1_000_000; // 0.01 BTC
    let buy_price = 55_000 * 1_000_000_000_u64; // 55,000 USDC per BTC

    let buy_order_call = market
        .get_instance()
        .methods()
        .open_order(buy_order_amount, buy_order_type, buy_price)
        .call_params(CallParameters::default())
        .unwrap();
    multi_call_handler = multi_call_handler.add_call(buy_order_call);

    // Add Sell Order (simplified)
    let sell_order_type = OrderType::Sell;
    let sell_order_amount = 1_000_000; // 0.01 BTC
    let sell_price = 65_000 * 1_000_000_000_u64; // 65,000 USDC per BTC

    let sell_order_call = market
        .get_instance()
        .methods()
        .open_order(sell_order_amount, sell_order_type, sell_price)
        .call_params(CallParameters::default())
        .unwrap();
    multi_call_handler = multi_call_handler.add_call(sell_order_call);

    // Execute multicall
    let multicall_tx_result = multi_call_handler.submit().await?;
    println!("Multicall transaction result: {:?}", multicall_tx_result);

    // Fetch and print user orders
    let orders = market.user_orders(wallet_id).await?.value;
    println!("Number of Orders: {:?}", orders.len());

    Ok(())
}

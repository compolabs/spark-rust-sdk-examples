use dotenv::dotenv;
use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    prelude::CallParameters,
    programs::calls::CallHandler,
    types::{AssetId, ContractId, Identity},
};
use spark_market_sdk::{OrderType, SparkMarketContract};
use std::{env, error::Error, str::FromStr};
use tokio::time::{sleep, Duration};

pub fn format_value_with_decimals(value: f64, decimals: u32) -> u64 {
    (value * 10f64.powi(decimals as i32)) as u64
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id_str = env::var("ETH_USDC_CONTRACT_ID")?;
    let eth_id_str: String = env::var("ETH_ID")?;
    let usdc_id_str: String = env::var("USDC_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id_str)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("Wallet Address: {:?}", main_wallet.address().to_string());

    // Depositing Assets
    let eth_id = AssetId::from_str(&eth_id_str)?;
    let usdc_id = AssetId::from_str(&usdc_id_str)?;

    // Fetch the current price of Ethereum from an API
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
    let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;
    let current_price: f64 = response["ethereum"]["usd"].as_f64().unwrap();
    println!("Current ETH price: ${:.2}", current_price);

    // Calculate total ETH and USDC needed for 100 $10 orders
    let num_orders = 100;
    let order_size_usd = 10.0; // $10 per order
    let total_usdc_needed = order_size_usd * num_orders as f64; // For buy orders
    let total_eth_needed = (order_size_usd / current_price) * num_orders as f64; // For sell orders

    // Deposit amounts
    let eth_amount = format_value_with_decimals(total_eth_needed, 9); // ETH has 9 decimals
    let usdc_amount = format_value_with_decimals(total_usdc_needed, 6); // USDC has 6 decimals

    // Print deposit amounts
    println!(
        "Depositing {:.6} ETH and ${:.2} USDC",
        format_to_readable_value(eth_amount, 9),
        format_to_readable_value(usdc_amount, 6)
    );

    // Deposit Calls
    let deposit_eth_call_params = CallParameters::new(eth_amount, eth_id, 20_000_000);
    let deposit_eth_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_eth_call_params)
        .unwrap();

    let deposit_usdc_call_params = CallParameters::new(usdc_amount, usdc_id, 20_000_000);
    let deposit_usdc_call = market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_usdc_call_params)
        .unwrap();

    // Execute the deposit multicall
    let mut deposit_multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
    deposit_multi_call_handler = deposit_multi_call_handler.add_call(deposit_eth_call);
    deposit_multi_call_handler = deposit_multi_call_handler.add_call(deposit_usdc_call);

    let deposit_tx_result = deposit_multi_call_handler.submit().await?;
    let deposit_tx_id = deposit_tx_result.tx_id();
    println!("Deposit transaction id: 0x{:?}", deposit_tx_id);

    sleep(Duration::from_secs(1)).await;

    // Initialize the multicall handler for orders
    let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
    let mut open_order_call_count = 0;
    let max_open_orders_per_call = 10; // Adjust if needed

    let eth_decimals = 9;
    let usdc_decimals = 6;

    let price_scaled = format_value_with_decimals(current_price, usdc_decimals);

    // Create 100 $10 ETH orders
    for i in 0..num_orders {
        // Alternate between buy and sell orders
        let order_type = if i % 2 == 0 {
            OrderType::Buy
        } else {
            OrderType::Sell
        };

        if order_type == OrderType::Buy {
            let buy_order_amount = format_value_with_decimals(order_size_usd, usdc_decimals);

            let buy_open_order_call = market
                .get_instance()
                .methods()
                .open_order(buy_order_amount, OrderType::Buy, price_scaled)
                .call_params(CallParameters::default())
                .unwrap();

            multi_call_handler = multi_call_handler.add_call(buy_open_order_call);
        } else {
            let eth_amount_order = order_size_usd / current_price;
            let sell_order_amount = format_value_with_decimals(eth_amount_order, eth_decimals);

            let sell_open_order_call = market
                .get_instance()
                .methods()
                .open_order(sell_order_amount, OrderType::Sell, price_scaled)
                .call_params(CallParameters::default())
                .unwrap();

            multi_call_handler = multi_call_handler.add_call(sell_open_order_call);
        }

        open_order_call_count += 1;

        // Submit orders in batches to avoid exceeding transaction size limits
        if open_order_call_count >= max_open_orders_per_call {
            let multicall_tx_result = multi_call_handler.submit().await?;
            let tx_id = multicall_tx_result.tx_id();
            println!(
                "Submitted {} orders. Transaction id: 0x{:?}",
                open_order_call_count, tx_id
            );

            // Reset the multicall handler and counter
            multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
            open_order_call_count = 0;

            sleep(Duration::from_secs(1)).await;
        }
    }

    // Submit any remaining orders
    if open_order_call_count > 0 {
        let multicall_tx_result = multi_call_handler.submit().await?;
        let tx_id = multicall_tx_result.tx_id();
        println!(
            "Submitted {} orders. Transaction id: 0x{:?}",
            open_order_call_count, tx_id
        );
    }

    sleep(Duration::from_secs(1)).await;

    let orders = market.user_orders(wallet_id.clone()).await?.value;
    println!("Total Number of Orders: {:?}", orders.len());

    let account = market.account(wallet_id.clone()).await?.value;
    println!(
        "Market account after deposits and order creation: {:?}",
        account
    );

    Ok(())
}

use dotenv::dotenv;
use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    crypto::SecretKey,
    programs::calls::CallHandler,
    types::{AssetId, ContractId},
};
use spark_market_sdk::{OrderType, SparkMarketContract};
use std::{env, error::Error, str::FromStr};

const ETH_PER_ORDER: u64 = 1_000_000; // 0.001 ETH
const SELL_ORDER_START_PRICE: u64 = 4400_000_000_000;
const SELL_ORDER_ITERATIONS: u64 = 0;
const SELL_ORDER_STEP: u64 = 100_000_000_000;

const USDC_PER_ORDER: u64 = 1_000_000; // 1 USDC
const BUY_ORDER_START_PRICE: u64 = 1000_000_000_000;
const BUY_ORDER_ITERATIONS: u64 = 5;
const BUY_ORDER_STEP: u64 = 100_000_000_000;

const SELL_ORDERS_COUNT: usize = SELL_ORDER_ITERATIONS as usize;
const BUY_ORDERS_COUNT: usize = BUY_ORDER_ITERATIONS as usize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let private_key = env::var("PRIVATE_KEY")?;
    let contract_id = env::var("ETH_USDC_CONTRACT_ID")?;
    let eth_id = env::var("ETH_ID")?;
    let usdc_id = env::var("USDC_ID")?;

    // Connect to provider
    // let provider_url = env::var("PROVIDER")?;
    let provider_url = "mainnet.fuel.network";

    let provider = Provider::connect(provider_url).await?;

    let private_key = SecretKey::from_str(&private_key).unwrap();
    let wallet = WalletUnlocked::new_from_private_key(private_key, Some(provider.clone()));

    let contract_id = ContractId::from_str(&contract_id)?;
    let market = SparkMarketContract::new(contract_id.clone(), wallet.clone()).await;

    let eth_id = AssetId::from_str(&eth_id)?;
    let usdc_id = AssetId::from_str(&usdc_id)?;

    // Calculate total amounts needed
    let total_eth_needed = ETH_PER_ORDER * (SELL_ORDERS_COUNT as u64);
    let total_usdc_needed = USDC_PER_ORDER * (BUY_ORDERS_COUNT as u64);

    // Check current balance and deposit only if needed
    let balance = market.account(wallet.address().into()).await?.value;
    // println!("Current balance: {:#?}", balance);

    let eth_balance = balance.liquid.base;
    let usdc_balance = balance.liquid.quote;

    let eth_needed = if eth_balance < total_eth_needed {
        total_eth_needed - eth_balance
    } else {
        0
    };

    let usdc_needed = if usdc_balance < total_usdc_needed {
        total_usdc_needed - usdc_balance
    } else {
        0
    };

    if eth_needed > 0 || usdc_needed > 0 {
        let mut multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        if eth_needed > 0 {
            let deposit_eth_call = market.deposit_call_handler(eth_needed, eth_id).await;
            multi_call_handler = multi_call_handler.add_call(deposit_eth_call);
        }

        if usdc_needed > 0 {
            let deposit_usdc_call = market.deposit_call_handler(usdc_needed, usdc_id).await;
            multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);
        }

        println!(
            "Depositing {} ETH and {} USDC",
            eth_needed as f64 / 1e9,
            usdc_needed as f64 / 1e6
        );
        let _deposit_result = multi_call_handler.submit().await?;
        println!(
            "Deposited {} ETH and {} USDC",
            eth_needed as f64 / 1e9,
            usdc_needed as f64 / 1e6
        );
    } else {
        println!("Sufficient balance available, no deposit needed");
    }

    if SELL_ORDER_ITERATIONS > 0 {
        // Create orders using multicall
        let mut order_multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        // Add sell orders to multicall
        for i in 0..SELL_ORDER_ITERATIONS {
            let price = SELL_ORDER_START_PRICE + i * SELL_ORDER_STEP;

            let sell_order_call = market
                .open_order_call_handler(ETH_PER_ORDER, OrderType::Sell, price)
                .await;

            order_multi_call_handler = order_multi_call_handler.add_call(sell_order_call);
            println!(
                "Added sell order to multicall: {} ETH at price {}",
                ETH_PER_ORDER as f64 / 1e9,
                price as f64 / 1e9
            );
        }
        let _order_result = order_multi_call_handler.submit().await.unwrap();
        println!("Successfully created {} sell orders", SELL_ORDER_ITERATIONS);
    }

    if BUY_ORDER_ITERATIONS > 0 {
        let mut order_multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        // Add buy orders to multicall
        for i in 0..BUY_ORDER_ITERATIONS {
            let price = BUY_ORDER_START_PRICE - i * BUY_ORDER_STEP;

            let buy_order_call = market
                .open_order_call_handler(USDC_PER_ORDER, OrderType::Buy, price)
                .await;

            order_multi_call_handler = order_multi_call_handler.add_call(buy_order_call);
            println!(
                "Added buy order to multicall: {} USDC at price {}",
                USDC_PER_ORDER as f64 / 1e6,
                price as f64 / 1e9
            );
        }
        // Submit all orders in one transaction
        let _order_result = order_multi_call_handler.submit().await.unwrap();
        println!("Successfully created {} buy orders", BUY_ORDER_ITERATIONS);
    }

    Ok(())
}

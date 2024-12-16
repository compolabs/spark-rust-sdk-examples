use dotenv::dotenv;
use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    crypto::SecretKey,
    programs::calls::CallHandler,
    types::{AssetId, Bits256, ContractId},
};
use spark_market_sdk::{OrderType, SparkMarketContract};
use std::{env, error::Error, str::FromStr};

const USDC_PER_ORDER: u64 = 1_000_000; // 1 USDC
const SELL_ORDER_START_PRICE: u64 = 1090_000_000; // 1 USDT per USDC
const SELL_ORDER_ITERATIONS: u64 = 1;
const SELL_ORDER_STEP: u64 = 10000_000; // 0.01 USDT step

const USDT_PER_ORDER: u64 = 1_000_000; // 1 USDT
const BUY_ORDER_START_PRICE: u64 = 950_000_000; // 0.99 USDT per USDC
const BUY_ORDER_ITERATIONS: u64 = 1;
const BUY_ORDER_STEP: u64 = 10000_000; // 0.01 USDT step

const SELL_ORDERS_COUNT: usize = SELL_ORDER_ITERATIONS as usize;
const BUY_ORDERS_COUNT: usize = BUY_ORDER_ITERATIONS as usize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let private_key = env::var("PRIVATE_KEY")?;
    let contract_id = env::var("USDC_USDT_CONTRACT_ID")?;
    let usdc_id = env::var("USDC_ID")?;
    let usdt_id = env::var("USDT_ID")?;

    // Connect to provider
    // let provider_url = env::var("PROVIDER")?;
    let provider_url = "mainnet.fuel.network";

    let provider = Provider::connect(provider_url).await?;

    let private_key = SecretKey::from_str(&private_key).unwrap();
    let wallet = WalletUnlocked::new_from_private_key(private_key, Some(provider.clone()));
    println!("Wallet address: {:?}", wallet.address());
    let contract_id = ContractId::from_str(&contract_id)?;
    let market = SparkMarketContract::new(contract_id.clone(), wallet.clone()).await;
    // let orders = market.user_orders(wallet.address().into()).await.unwrap().value;
    // println!("Orders: {:#?}", orders);
    // return Ok(());

    let usdc_id = AssetId::from_str(&usdc_id)?;
    let usdt_id = AssetId::from_str(&usdt_id)?;

    // Calculate total amounts needed
    let total_usdc_needed = USDC_PER_ORDER * (SELL_ORDERS_COUNT as u64);
    let total_usdt_needed = USDT_PER_ORDER * (BUY_ORDERS_COUNT as u64);

    // Check current balance and deposit only if needed
    let balance = market.account(wallet.address().into()).await?.value;
    // println!("Current balance: {:#?}", balance);

    let usdc_balance = balance.liquid.base;
    let usdt_balance = balance.liquid.quote;

    let usdc_needed = if usdc_balance < total_usdc_needed {
        total_usdc_needed - usdc_balance
    } else {
        0
    };

    let usdt_needed = if usdt_balance < total_usdt_needed {
        total_usdt_needed - usdt_balance
    } else {
        0
    };

    if usdc_needed > 0 || usdt_needed > 0 {
        let mut multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        if usdc_needed > 0 {
            let deposit_usdc_call = market.deposit_call_handler(usdc_needed, usdc_id);
            multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);
        }

        if usdt_needed > 0 {
            let deposit_usdt_call = market.deposit_call_handler(usdt_needed, usdt_id);
            multi_call_handler = multi_call_handler.add_call(deposit_usdt_call);
        }

        println!(
            "Depositing {} USDC and {} USDT",
            usdc_needed as f64 / 1e6,
            usdt_needed as f64 / 1e6
        );
        let _deposit_result = multi_call_handler.submit().await?;
        println!(
            "Deposited {} USDC and {} USDT",
            usdc_needed as f64 / 1e6,
            usdt_needed as f64 / 1e6
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

            let sell_order_call =
                market.open_order_call_handler(USDC_PER_ORDER, OrderType::Sell, price);

            order_multi_call_handler = order_multi_call_handler.add_call(sell_order_call);
            println!(
                "Added sell order to multicall: {} USDC at price {}",
                USDC_PER_ORDER as f64 / 1e6,
                price as f64 / 1e9
            );
        }
        order_multi_call_handler.call::<Vec<Bits256>>().await?;
        println!("Successfully created {} sell orders", SELL_ORDER_ITERATIONS);
    }

    if BUY_ORDER_ITERATIONS > 0 {
        let mut order_multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        // Add buy orders to multicall
        for i in 0..BUY_ORDER_ITERATIONS {
            let price = BUY_ORDER_START_PRICE - i * BUY_ORDER_STEP;

            let buy_order_call =
                market.open_order_call_handler(USDT_PER_ORDER, OrderType::Buy, price);

            order_multi_call_handler = order_multi_call_handler.add_call(buy_order_call);
            println!(
                "Added buy order to multicall: {} USDT at price {}",
                USDT_PER_ORDER as f64 / 1e6,
                price as f64 / 1e9
            );
        }
        // Submit all orders in one transaction
        order_multi_call_handler.call::<Vec<Bits256>>().await?;
        println!("Successfully created {} buy orders", BUY_ORDER_ITERATIONS);
    }

    Ok(())
}

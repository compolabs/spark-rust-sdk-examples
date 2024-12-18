
use dotenv::dotenv;
use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    crypto::SecretKey,
    programs::calls::CallHandler,
    types::{AssetId, ContractId},
};
use spark_market_sdk::{OrderType, SparkMarketContract};
use rand::Rng;

use std::{env, error::Error, str::FromStr};

const FUEL_PER_ORDER: u64 = 1_000_000; // 0.001 FUEL
const SELL_ORDER_START_PRICE: u64 = 4400_000_000_000;
const SELL_ORDER_ITERATIONS: u64 = 0; // 0 iterations for debugging
const SELL_ORDER_STEP_PERCENT: f64 = 0.002; // 0.2%

const USDC_PER_ORDER: u64 = 300_000; // 1 USDC
const BUY_ORDER_START_PRICE: u64 = 18_000_000; // 0.02 USDC per 1 FUEL
const BUY_ORDER_ITERATIONS: u64 = 10;
const BUY_ORDER_STEP_PERCENT: f64 = 0.1;

const SELL_ORDERS_COUNT: usize = SELL_ORDER_ITERATIONS as usize;
const BUY_ORDERS_COUNT: usize = BUY_ORDER_ITERATIONS as usize;

const VOLUME_VARIATION_PERCENT: f64 = 0.1; // 10%


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let private_key = env::var("PRIVATE_KEY")?;
    let contract_id = env::var("FUEL_USDC_CONTRACT_ID")?;
    let fuel_id = env::var("FUEL_ID")?;
    let usdc_id = env::var("USDC_ID")?;

    // Connect to provider
    let provider_url = "mainnet.fuel.network";
    let provider = Provider::connect(provider_url).await?;

    let private_key = SecretKey::from_str(&private_key).unwrap();
    let wallet = WalletUnlocked::new_from_private_key(private_key, Some(provider.clone()));

    let contract_id = ContractId::from_str(&contract_id)?;
    let market = SparkMarketContract::new(contract_id.clone(), wallet.clone()).await;

    let fuel_id = AssetId::from_str(&fuel_id)?;
    let usdc_id = AssetId::from_str(&usdc_id)?;

    // Calculate total amounts needed
    let total_fuel_needed = FUEL_PER_ORDER * (SELL_ORDERS_COUNT as u64);
    let total_usdc_needed = USDC_PER_ORDER * (BUY_ORDERS_COUNT as u64);

    // Check current balance and deposit only if needed
    let balance = market.account(wallet.address().into()).await?.value;

    let fuel_balance = balance.liquid.base;
    let usdc_balance = balance.liquid.quote;

    let fuel_needed = if fuel_balance < total_fuel_needed {
        total_fuel_needed - fuel_balance
    } else {
        0
    };

    let usdc_needed = if usdc_balance < total_usdc_needed {
        total_usdc_needed - usdc_balance
    } else {
        0
    };

    if fuel_needed > 0 || usdc_needed > 0 {
        let mut multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        if fuel_needed > 0 {
            let deposit_fuel_call = market.deposit_call_handler(fuel_needed, fuel_id);
            multi_call_handler = multi_call_handler.add_call(deposit_fuel_call);
        }

        if usdc_needed > 0 {
            let deposit_usdc_call = market.deposit_call_handler(usdc_needed, usdc_id);
            multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);
        }

        println!(
            "Depositing {} FUEL and {} USDC",
            fuel_needed as f64 / 1e9,
            usdc_needed as f64 / 1e6
        );
        let _deposit_result = multi_call_handler.submit().await?;
        println!(
            "Deposited {} FUEL and {} USDC",
            fuel_needed as f64 / 1e9,
            usdc_needed as f64 / 1e6
        );
    } else {
        println!("Sufficient balance available, no deposit needed");
    }

    if SELL_ORDER_ITERATIONS > 0 {
        let mut order_multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        for i in 0..SELL_ORDER_ITERATIONS {
            let price = (SELL_ORDER_START_PRICE as f64 * (1.0 + i as f64 * SELL_ORDER_STEP_PERCENT)) as u64;

            let sell_order_call =
                market.open_order_call_handler(FUEL_PER_ORDER, OrderType::Sell, price);

            order_multi_call_handler = order_multi_call_handler.add_call(sell_order_call);
            println!(
                "Added sell order to multicall: {} FUEL at price {}",
                FUEL_PER_ORDER as f64 / 1e9,
                price as f64 / 1e9
            );
        }
        let _order_result = order_multi_call_handler.submit().await.unwrap();
        println!("Successfully created {} sell orders", SELL_ORDER_ITERATIONS);
    }

    if BUY_ORDER_ITERATIONS > 0 {
        let mut order_multi_call_handler = CallHandler::new_multi_call(wallet.clone());

        for i in 0..BUY_ORDER_ITERATIONS {
            let price = (BUY_ORDER_START_PRICE as f64 * (1.0 - i as f64 * BUY_ORDER_STEP_PERCENT)) as u64;
            println!("price {:?}", price);
            // Генерируем вариацию объёма в пределах 1 USDC ± 10%
            let mut rng = rand::thread_rng();
            let volume_variation = 1.0 + rng.gen_range(-VOLUME_VARIATION_PERCENT..VOLUME_VARIATION_PERCENT);
            let adjusted_usdc_volume = (USDC_PER_ORDER as f64 * volume_variation) as u64;
            println!("adjusted_usdc_volume {:?}", adjusted_usdc_volume);

            // Рассчитываем объём FUEL, который можно купить за указанную сумму USDC по текущей цене
            let adjusted_fuel_volume = (adjusted_usdc_volume as f64 * 1e9 / price as f64) as u64;
            let a2 = adjusted_fuel_volume * 1000;
            println!("adjusted_fuel_volume {:?}", a2);
            
            let buy_order_call =
                market.open_order_call_handler(a2, OrderType::Buy, price);

            order_multi_call_handler = order_multi_call_handler.add_call(buy_order_call);
            println!(
                "Added buy order to multicall: {} FUEL at price {} ({} USDC)",
                adjusted_fuel_volume as f64 / 1e9,
                price as f64 / 1e9,
                adjusted_usdc_volume as f64 / 1e6
            );
        }
        let order_result = order_multi_call_handler.submit().await.unwrap();
        println!("order_result {:?}", order_result);
        println!("Successfully created {} buy orders", BUY_ORDER_ITERATIONS);
    }

    Ok(())
}

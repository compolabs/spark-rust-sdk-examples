use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked, ViewOnlyAccount},
    crypto::SecretKey,
    programs::calls::CallHandler,
    types::{AssetId, ContractId, Identity},
};
use std::str::FromStr;
use rand::Rng;

use spark_market_sdk::{OrderType, SparkMarketContract};
use std::error::Error;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readble_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

const START_PRICE: u64 = 5_000_000_000;
const MIN_PRICE_STEP: u64 = 10_000;
const MAX_PRICE_STEP: u64 = 10_000_000;
const MIN_AMOUNT: u64 = 10_000_000;
const MAX_AMOUNT: u64 = 1_000_000_000;
const ORDER_COUNT: u64 = 1000;
const MARKET_CONTRACT: &str = "0x12a5f8666279f841e5900500297ce3c8bcf40103dd191c56dd3ec86f92b9217b";
const BASE_TOKEN: &str = "0x0b2d808a898cdae8b8661d398a98f8ff45e1e0f536ba2e498f6c7e53a71932cd";
const QUOTE_TOKEN: &str = "0x368f9275e7d072794527b57d5b54688300008a400f41d926a013195e7074029c";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let pk = env::var("PRIVATE_KEY")?;

    // Connect to provider
    let provider = Provider::connect("mainnet.fuel.network").await?;

    let main_wallet = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&pk).unwrap(),
        Some(provider.clone()),
    );
    let contract_id = ContractId::from_str(MARKET_CONTRACT)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    let base_id: AssetId = AssetId::from_str(BASE_TOKEN)?;
    let quote_id: AssetId = AssetId::from_str(QUOTE_TOKEN)?;

    // Getting asset balances
    let account = market.account(wallet_id).await.unwrap().value;
    println!("account balance: {:?}", account);
    // Get base token balance
    let base_balance = main_wallet.get_asset_balance(&base_id).await.unwrap();
    if base_balance > 0 {
        println!(
            "Depositing {} BASE",
            format_to_readble_value(base_balance, 9)
        );
        match market.deposit(base_balance, base_id).await {
            Ok(_) => {
                println!("Base Deposit Success");
                Ok(())
            }
            Err(e) => {
                print!("Base Deposit error: {:?}", e);
                Err(e)
            }
        }
        .unwrap();
    }

    // Get quote token balance
    let quote_balance = main_wallet.get_asset_balance(&quote_id).await.unwrap();
    if quote_balance > 0 {
        println!(
            "Depositing {} QUOTE",
            format_to_readble_value(quote_balance, 9)
        );
        match market.deposit(quote_balance, quote_id).await {
            Ok(_) => {
                println!("Quote Deposit Success");
                Ok(())
            }
            Err(e) => {
                print!("Quote Deposit error: {:?}", e);
                Err(e)
            }
        }
        .unwrap();
    }

    let mut rng = rand::thread_rng();

    // Iterate through ORDER_COUNT steps from START_PRICE
    for i in 0..ORDER_COUNT {
        let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());

        // Random price step and amounts for each iteration
        let price_step = rng.gen_range(MIN_PRICE_STEP..=MAX_PRICE_STEP);
        let base_amount = rng.gen_range(MIN_AMOUNT..=MAX_AMOUNT);
        let quote_amount = rng.gen_range(MIN_AMOUNT..=MAX_AMOUNT);

        // Buy Order - decreasing price
        let buy_price = START_PRICE - (i * price_step);

        let order_type: OrderType = OrderType::Buy;
        println!(
            "Opening Buy Order: {} BASE at {} BASE/QUOTE",
            format_to_readble_value(base_amount, 9),
            format_to_readble_value(buy_price, 9)
        );

        let buy_order_call = market
            .open_order_call_handler(base_amount, order_type, buy_price)
            .await;
        multi_call_handler = multi_call_handler.add_call(buy_order_call);

        // Sell Order - increasing price
        let sell_price = START_PRICE + (i * price_step);

        let order_type: OrderType = OrderType::Sell;
        println!(
            "Opening Sell Order: {} BASE at {} BASE/QUOTE",
            format_to_readble_value(quote_amount, 9),
            format_to_readble_value(sell_price, 9)
        );

        let sell_order_call = market
            .open_order_call_handler(quote_amount, order_type, sell_price)
            .await;
        multi_call_handler = multi_call_handler.add_call(sell_order_call);

        match multi_call_handler.submit().await {
            Ok(_) => println!("Multicall orders submitted successfully"),
            Err(e) => println!("Multicall orders error: {:?}", e),
        }
    }

    let orders = market.user_orders(wallet_id).await?.value;
    println!("orders {:?}", orders);

    Ok(())
}

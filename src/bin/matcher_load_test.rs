use dotenv::dotenv;
use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    prelude::CallParameters,
    programs::calls::CallHandler,
    types::{AssetId, ContractId, Identity},
};
use spark_market_sdk::{OrderType, SparkMarketContract};
use std::{env, error::Error, str::FromStr};
// Removed unnecessary imports from rand_distr
use tokio::time::{sleep, Duration};

pub fn format_value_with_decimals(value: f64, decimals: u32) -> u64 {
    (value * 10f64.powi(decimals as i32)) as u64
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

// Function to compute the normal distribution PDF
fn normal_pdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    let variance = std_dev * std_dev;
    let denom = (2.0 * std::f64::consts::PI * variance).sqrt();
    let num = (-((x - mean).powi(2)) / (2.0 * variance)).exp();
    num / denom
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id_str = env::var("BTC_USDC_CONTRACT_ID")?;
    let btc_id_str: String = env::var("BTC_ID")?;
    let usdc_id_str: String = env::var("USDC_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id_str)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    // Initialize iteration counter
    let mut iteration = 0;

    loop {
        iteration += 1;
        println!("\nStarting iteration {}", iteration);

        // Depositing Assets
        let btc_id = AssetId::from_str(&btc_id_str)?;
        let btc_amount = format_value_with_decimals(1.0, 8);

        let usdc_id = AssetId::from_str(&usdc_id_str)?;
        let usdc_amount = format_value_with_decimals(3000.0, 6);

        let account = market.account(wallet_id.clone()).await?.value;
        println!(
            "Market account before deposit and order creation: {:?}",
            account
        );

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

        // Execute the deposit multicall
        let mut deposit_multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
        deposit_multi_call_handler = deposit_multi_call_handler.add_call(deposit_btc_call);
        deposit_multi_call_handler = deposit_multi_call_handler.add_call(deposit_usdc_call);

        let deposit_tx_result = deposit_multi_call_handler.submit().await?;
        let deposit_tx_id = deposit_tx_result.tx_id();
        println!("Deposit transaction id: 0x{:?}", deposit_tx_id);

        sleep(Duration::from_secs(1)).await;

        let protocol_fee = market.protocol_fee().await?.value;
        println!("Protocol fee: {:?}", protocol_fee);

        // Get the current price of Bitcoin from an API
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
        let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;
        let mut current_price: f64 = response["bitcoin"]["usd"].as_f64().unwrap();

        // Adjust the price based on iteration
        if iteration % 2 == 1 {
            // Odd iteration: reduce price by 2%
            current_price *= 0.98;
            println!(
                "Adjusted BTC price for iteration {}: ${:.2}",
                iteration, current_price
            );
        } else {
            println!(
                "Current BTC price for iteration {}: ${:.2}",
                iteration, current_price
            );
        }

        // Define the price range
        let price_range = 5000.0;
        let price_step = 100.0; // Price levels every $100
        let num_levels = ((price_range * 2.0) / price_step) as usize + 1;

        // Generate price levels within ±$5000 of the current price
        let mut price_levels = Vec::with_capacity(num_levels);

        let start_price = current_price - price_range;
        for i in 0..num_levels {
            let price = start_price + i as f64 * price_step;
            price_levels.push(price);
        }

        // Use normal distribution to assign liquidity
        let mean = current_price;
        let std_dev = price_range / 3.0; // Standard deviation set to cover ±3σ within the range

        // Compute the probability density for each price level
        let mut liquidity_weights = Vec::with_capacity(num_levels);
        let mut total_weight = 0.0;
        for price in &price_levels {
            let weight = normal_pdf(*price, mean, std_dev);
            liquidity_weights.push(weight);
            total_weight += weight;
        }

        // Normalize the weights to determine order sizes
        let total_available_btc = 0.15; // Total BTC to allocate for sell orders
        let total_available_usdc = 10000.0; // Total USDC to allocate for buy orders

        let btc_decimals = 8;
        let usdc_decimals = 6;

        // Initialize the multicall handler and counter
        let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
        let mut open_order_call_count = 0;
        let max_open_orders_per_call = 10;

        for i in 0..num_levels {
            let price = price_levels[i];
            let weight = liquidity_weights[i];

            // Calculate order sizes proportional to the liquidity weights
            let sell_order_amount_btc = total_available_btc * (weight / total_weight);
            let buy_order_amount_usdc = total_available_usdc * (weight / total_weight);

            // Skip negligible order sizes
            if sell_order_amount_btc < 0.0001 && buy_order_amount_usdc < 1.0 {
                continue;
            }

            // Create Sell Orders (selling BTC for USDC)
            if sell_order_amount_btc >= 0.0001 {
                let sell_order_amount =
                    format_value_with_decimals(sell_order_amount_btc, 8);
                let sell_price_scaled = format_value_with_decimals(price, 9);

                let sell_open_order_call = market
                    .get_instance()
                    .methods()
                    .open_order(sell_order_amount, OrderType::Sell, sell_price_scaled)
                    .call_params(CallParameters::default())
                    .unwrap();

                multi_call_handler = multi_call_handler.add_call(sell_open_order_call);
                open_order_call_count += 1;

                if open_order_call_count >= max_open_orders_per_call {
                    // Submit the multicall
                    let multicall_tx_result = multi_call_handler.submit().await?;
                    let tx_id = multicall_tx_result.tx_id();
                    println!("Multicall transaction id: 0x{:?}", tx_id);

                    // Reset the multicall handler and counter
                    multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
                    open_order_call_count = 0;

                    sleep(Duration::from_secs(1)).await;
                }
            }

            // Create Buy Orders (buying BTC with USDC)
            if buy_order_amount_usdc >= 1.0 {
                let buy_order_amount =
                    format_value_with_decimals(buy_order_amount_usdc, 7);
                let buy_price_scaled = format_value_with_decimals(price, 9);

                let buy_open_order_call = market
                    .get_instance()
                    .methods()
                    .open_order(buy_order_amount, OrderType::Buy, buy_price_scaled)
                    .call_params(CallParameters::default())
                    .unwrap();

                multi_call_handler = multi_call_handler.add_call(buy_open_order_call);
                open_order_call_count += 1;

                if open_order_call_count >= max_open_orders_per_call {
                    // Submit the multicall
                    let multicall_tx_result = multi_call_handler.submit().await?;
                    let tx_id = multicall_tx_result.tx_id();
                    println!("Multicall transaction id: 0x{:?}", tx_id);

                    // Reset the multicall handler and counter
                    multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
                    open_order_call_count = 0;

                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        // Submit any remaining open order calls
        if open_order_call_count > 0 {
            let multicall_tx_result = multi_call_handler.submit().await?;
            let tx_id = multicall_tx_result.tx_id();
            println!("Multicall transaction id: 0x{:?}", tx_id);
        }

        sleep(Duration::from_secs(1)).await;

        let orders = market.user_orders(wallet_id.clone()).await?.value;
        println!("Number of Orders: {:?}", orders.len());

        let account = market.account(wallet_id.clone()).await?.value;
        println!(
            "Market account after deposit and order creation: {:?}",
            account
        );

        // Wait 30 seconds before starting the next iteration
        println!("Waiting 30 seconds before the next iteration...");
        sleep(Duration::from_secs(30)).await;
    }
}

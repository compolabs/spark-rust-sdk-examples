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
    (value * 10f64.powi(decimals as i32)).round() as u64
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

    // Initialize iteration counter
    let mut iteration = 0;

    loop {
        iteration += 1;
        println!("\nStarting iteration {}", iteration);

        // Depositing Assets
        let eth_id = AssetId::from_str(&eth_id_str)?;
        let eth_decimals = 9; // ETH has 9 decimals
        let total_available_eth = 1.0; // Total ETH to allocate for sell orders
        let eth_amount = format_value_with_decimals(total_available_eth, eth_decimals);

        let usdc_id = AssetId::from_str(&usdc_id_str)?;
        let usdc_decimals = 6; // USDC has 6 decimals
        let total_available_usdc = 3000.0; // Total USDC to allocate for buy orders
        let usdc_amount = format_value_with_decimals(total_available_usdc, usdc_decimals);

        let account = market.account(wallet_id.clone()).await?.value;
        println!(
            "Market account before deposit and order creation: {:?}",
            account
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

        let protocol_fee = market.protocol_fee().await?.value;
        println!("Protocol fee: {:?}", protocol_fee);

        // Get the current price of Ethereum from an API
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
        let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;
        let mut current_price: f64 = response["ethereum"]["usd"].as_f64().unwrap();

        // Adjust the price based on iteration
        if iteration % 2 == 1 {
            // Odd iteration: reduce price by 2%
            current_price *= 0.98;
            println!(
                "Adjusted ETH price for iteration {}: ${:.2}",
                iteration, current_price
            );
        } else {
            println!(
                "Current ETH price for iteration {}: ${:.2}",
                iteration, current_price
            );
        }

        // Define the price range
        let price_range = 500.0;
        let price_step = 10.0; // Price levels every $10
        let num_levels = ((price_range * 2.0) / price_step) as usize + 1;

        // Generate price levels within ±$500 of the current price
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

        // Set the desired order size
        let desired_order_size_eth = 0.01; // 0.01 ETH per order

        // Initialize the multicall handler and counter
        let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
        let mut open_order_call_count = 0;
        let max_open_orders_per_call = 10;

        for i in 0..num_levels {
            let price = price_levels[i];
            let weight = liquidity_weights[i];

            // Calculate sell order amount (fixed at desired_order_size_eth)
            let sell_order_amount_eth = desired_order_size_eth;
            let sell_order_amount_scaled =
                format_value_with_decimals(sell_order_amount_eth, eth_decimals);

            // Calculate buy order amount in USDC equivalent to desired_order_size_eth
            let buy_order_amount_usdc = desired_order_size_eth * price;
            let buy_order_amount_scaled =
                format_value_with_decimals(buy_order_amount_usdc, usdc_decimals);

            // Adjust buy and sell prices with a small spread if needed
            let spread = 0.001; // 0.1% spread
            let half_spread = spread / 2.0; // 0.05%

            // Adjust sell price up by half the spread
            let sell_price = price * (1.0 + half_spread);
            let sell_price_scaled = format_value_with_decimals(sell_price, 8); // Price scaled with 8 decimals

            // Adjust buy price down by half the spread
            let buy_price = price * (1.0 - half_spread);
            let buy_price_scaled = format_value_with_decimals(buy_price, 8); // Price scaled with 8 decimals

            // Create Sell Orders (selling ETH for USDC)
            if sell_order_amount_eth * (weight / total_weight) >= 0.0001 {
                let sell_open_order_call = market
                    .get_instance()
                    .methods()
                    .open_order(
                        sell_order_amount_scaled,
                        OrderType::Sell,
                        sell_price_scaled * 10,
                    )
                    .call_params(CallParameters::default())
                    .unwrap();

                multi_call_handler = multi_call_handler.add_call(sell_open_order_call);
                open_order_call_count += 1;

                if open_order_call_count >= max_open_orders_per_call {
                    // Submit the multicall
                    let multicall_tx_result = multi_call_handler.submit().await?;
                    let tx_id = multicall_tx_result.tx_id();
                    println!(
                        "Submitted {} sell orders. Transaction id: 0x{:?}",
                        open_order_call_count, tx_id
                    );

                    // Reset the multicall handler and counter
                    multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());
                    open_order_call_count = 0;

                    sleep(Duration::from_secs(1)).await;
                }
            }

            // Create Buy Orders (buying ETH with USDC)
            if buy_order_amount_usdc * (weight / total_weight) >= 1.0 {
                let buy_open_order_call = market
                    .get_instance()
                    .methods()
                    .open_order(
                        buy_order_amount_scaled,
                        OrderType::Buy,
                        buy_price_scaled * 10,
                    )
                    .call_params(CallParameters::default())
                    .unwrap();

                multi_call_handler = multi_call_handler.add_call(buy_open_order_call);
                open_order_call_count += 1;

                if open_order_call_count >= max_open_orders_per_call {
                    // Submit the multicall
                    let multicall_tx_result = multi_call_handler.submit().await?;
                    let tx_id = multicall_tx_result.tx_id();
                    println!(
                        "Submitted {} buy orders. Transaction id: 0x{:?}",
                        open_order_call_count, tx_id
                    );

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
            println!(
                "Submitted {} orders. Transaction id: 0x{:?}",
                open_order_call_count, tx_id
            );
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

use dotenv::dotenv;
use std::{env, error::Error, str::FromStr};

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    types::{AssetId, Bits256, ContractId, Identity},
};

use spark_market_sdk::{OrderType, SparkMarketContract};

use tokio::time::{sleep, Duration};

pub fn format_value_with_decimals(value: f64, decimals: u32) -> u64 {
    (value * 10f64.powi(decimals as i32)).round() as u64
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

// Helper function to convert Bits256 to hex string
fn hex_str_from_bits256(bits: &Bits256) -> String {
    format!("0x{}", hex::encode(bits.0))
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
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::connect(provider_url).await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id_str)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("Wallet Address: {:?}", main_wallet.address().to_string());

    // Define the total value of orders to open
    let total_order_value_usd = 100.0; // Total value in USD

    // Start of single execution block
    {
        println!("\nStarting new iteration...");

        // Fetch the current ETH price from CoinGecko
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
        let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;
        let current_price: f64 = response["ethereum"]["usd"].as_f64().unwrap();
        println!("Current ETH price: ${:.2}", current_price);

        // Define the ±1.5% price range
        let lower_bound = current_price * 0.985; // 1 - 0.015
        let upper_bound = current_price * 1.015; // 1 + 0.015
        println!(
            "Order price range: ${:.2} - ${:.2}",
            lower_bound, upper_bound
        );

        // Fetch user's existing orders
        let orders = market.user_orders(wallet_id.clone()).await?.value;
        println!("Number of existing orders: {}", orders.len());

        // Collect orders to cancel
        let mut orders_to_cancel = Vec::new();

        for order in &orders {
            // Fetch order details
            let order_details = market.order(*order).await?.value.unwrap();

            // Convert order price to f64
            let order_price_scaled = order_details.price;
            let order_price = format_to_readable_value(order_price_scaled, 9); // Price is in base 1e9

            // Check if the order price is outside the ±1.5% range
            if order_price < lower_bound || order_price > upper_bound {
                orders_to_cancel.push(*order);
            }
        }

        println!("Number of orders to cancel: {}", orders_to_cancel.len());

        // Cancel orders individually
        let total_cancel_orders = orders_to_cancel.len();

        if total_cancel_orders > 0 {
            for order_id in orders_to_cancel {
                let order_id_hex = hex_str_from_bits256(&order_id);
                println!("Cancelling order {}", order_id_hex);
                match market.cancel_order(order_id).await {
                    Ok(_) => println!("Order {} cancelled successfully", order_id_hex),
                    Err(e) => println!("Error cancelling order {}: {:?}", order_id_hex, e),
                }
                sleep(Duration::from_millis(500)).await; // Slight delay between cancellations
            }
        } else {
            println!("No orders to cancel");
        }

        sleep(Duration::from_secs(1)).await;

        // Get asset balances
        let account = market.account(wallet_id.clone()).await?.value;
        let liquid_base = account.liquid.base;
        let liquid_quote = account.liquid.quote;

        println!("Raw account before deposit: {:?}", account);

        let eth_decimals = 9; // ETH has 9 decimals
        let usdc_decimals = 6; // USDC has 6 decimals

        let eth_balance = format_to_readable_value(liquid_base, eth_decimals);
        let usdc_balance = format_to_readable_value(liquid_quote, usdc_decimals);

        println!("Market ETH balance: {}", eth_balance);
        println!("Market USDC balance: {}", usdc_balance);

        // Define the number of price levels
        let num_levels = 5; // Number of price levels within the ±1.5% range

        // Calculate desired order size per order
        let desired_order_size_usd = total_order_value_usd / (num_levels as f64 * 2.0); // Since we have buy and sell orders at each level

        println!(
            "Desired order size per order: ${:.2}",
            desired_order_size_usd
        );

        // Generate price levels within the ±1.5% range
        let price_step = (upper_bound - lower_bound) / (num_levels as f64 - 1.0); // Adjusted for num_levels
        let mut price_levels = Vec::with_capacity(num_levels);

        for i in 0..num_levels {
            let price = lower_bound + i as f64 * price_step;
            price_levels.push(price);
        }

        // Calculate total required balances for new orders
        let mut total_required_eth = 0.0;
        let mut total_required_usdc = 0.0;

        // Store calculated order amounts for use in order placement
        let mut sell_order_amounts_eth = Vec::with_capacity(num_levels);
        let mut sell_prices_scaled = Vec::with_capacity(num_levels);
        let mut buy_order_amounts_eth = Vec::with_capacity(num_levels);
        let mut buy_prices_scaled = Vec::with_capacity(num_levels);

        for price in &price_levels {
            // Adjust buy and sell prices with a small spread if needed
            let spread = 0.0001; // 0.01% spread
            let half_spread = spread / 2.0; // 0.005%

            // Adjust sell price up by half the spread
            let sell_price = price * (1.0 + half_spread);
            let sell_price_scaled = format_value_with_decimals(sell_price, 9); // Price is in base 1e9
            sell_prices_scaled.push(sell_price_scaled);

            // Adjust buy price down by half the spread
            let buy_price = price * (1.0 - half_spread);
            let buy_price_scaled = format_value_with_decimals(buy_price, 9); // Price is in base 1e9
            buy_prices_scaled.push(buy_price_scaled);

            // Calculate sell order amount (amount of ETH equivalent to desired_order_size_usd at sell price)
            let sell_order_amount_eth = desired_order_size_usd / sell_price;
            sell_order_amounts_eth.push(sell_order_amount_eth);
            total_required_eth += sell_order_amount_eth;

            // Calculate buy order amount (amount of ETH equivalent to desired_order_size_usd at buy price)
            let buy_order_amount_eth = desired_order_size_usd / buy_price;
            buy_order_amounts_eth.push(buy_order_amount_eth);
            total_required_usdc += desired_order_size_usd; // Each buy order requires desired_order_size_usd USDC
        }

        println!(
            "Total required ETH for sell orders: {:.6}",
            total_required_eth
        );
        println!(
            "Total required USDC for buy orders: {:.2}",
            total_required_usdc
        );

        // Deposit ETH if necessary
        if eth_balance < total_required_eth {
            let deposit_amount_eth = total_required_eth - eth_balance;
            let deposit_amount_scaled =
                format_value_with_decimals(deposit_amount_eth, eth_decimals);

            println!("Depositing additional ETH: {:?}", deposit_amount_scaled);

            let eth_id = AssetId::from_str(&eth_id_str)?;

            // Perform the deposit
            println!("Depositing ETH");
            match market.deposit(deposit_amount_scaled, eth_id).await {
                Ok(_) => {
                    println!("ETH Deposit Success");
                    Ok(())
                }
                Err(e) => {
                    println!("ETH Deposit error: {:?}", e);
                    Err(e)
                }
            }
            .unwrap();
        } else {
            println!("No additional ETH deposit needed.");
        }

        // Deposit USDC if necessary
        if usdc_balance < total_required_usdc {
            let deposit_amount_usdc = total_required_usdc - usdc_balance;
            let deposit_amount_scaled =
                format_value_with_decimals(deposit_amount_usdc, usdc_decimals);

            println!("Depositing additional USDC: {:?}", deposit_amount_scaled);

            let usdc_id = AssetId::from_str(&usdc_id_str)?;

            // Perform the deposit
            println!("Depositing USDC");
            match market.deposit(deposit_amount_scaled, usdc_id).await {
                Ok(_) => {
                    println!("USDC Deposit Success");
                    Ok(())
                }
                Err(e) => {
                    println!("USDC Deposit error: {:?}", e);
                    Err(e)
                }
            }
            .unwrap();
        } else {
            println!("No additional USDC deposit needed.");
        }

        // Get asset balances after deposit
        let account = market.account(wallet_id.clone()).await?.value;
        println!("Raw account after deposit: {:?}", account);

        sleep(Duration::from_secs(1)).await;

        // Open orders individually
        for i in 0..num_levels {
            let sell_order_amount_eth = sell_order_amounts_eth[i];
            let sell_order_amount_scaled =
                format_value_with_decimals(sell_order_amount_eth, eth_decimals);

            let buy_order_amount_eth = buy_order_amounts_eth[i];
            let buy_order_amount_scaled =
                format_value_with_decimals(buy_order_amount_eth, eth_decimals);

            let sell_price_scaled = sell_prices_scaled[i];
            let buy_price_scaled = buy_prices_scaled[i];

            // Create Sell Orders (selling ETH for USDC)
            if sell_order_amount_eth >= 0.00001 {
                println!(
                    "Opening sell order:\nAmount: {:?}\nPrice: {:?}",
                    sell_order_amount_scaled, sell_price_scaled
                );

                match market
                    .open_order(sell_order_amount_scaled, OrderType::Sell, sell_price_scaled)
                    .await
                {
                    Ok(_) => println!("Sell order opened successfully"),
                    Err(e) => println!("Error opening sell order: {:?}", e),
                }

                sleep(Duration::from_secs(1)).await; // Wait between orders
            }

            // Create Buy Orders (buying ETH with USDC)
            if buy_order_amount_eth >= 0.00001 {
                println!(
                    "Opening buy order:\nAmount: {:?}\nPrice: {:?}",
                    buy_order_amount_scaled, buy_price_scaled
                );

                match market
                    .open_order(buy_order_amount_scaled, OrderType::Buy, buy_price_scaled)
                    .await
                {
                    Ok(_) => println!("Buy order opened successfully"),
                    Err(e) => println!("Error opening buy order: {:?}", e),
                }

                sleep(Duration::from_secs(1)).await; // Wait between orders
            }
        }

        sleep(Duration::from_secs(1)).await;

        // Fetch updated user's orders
        let orders = market.user_orders(wallet_id.clone()).await?.value;
        println!("Number of Orders after update: {:?}", orders.len());

        // Wait 30 seconds before starting the next iteration
        println!("Waiting 30 seconds before the next iteration...");
        sleep(Duration::from_secs(30)).await;
    }

    Ok(())
}

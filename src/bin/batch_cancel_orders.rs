use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, programs::calls::CallHandler,
    crypto::SecretKey,
    types::ContractId, types::Identity,
};
use std::str::FromStr;

use spark_market_sdk::SparkMarketContract;
use std::error::Error;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readble_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

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
    // Fuel wallet address
    let wallet_id: Identity = wallet.address().into();
    println!("wallet {:?}", wallet.address().to_string());

    // Fetching user orders
    let orders = market.user_orders(wallet_id).await?.value;
    println!("User Orders: {:?}", orders);

    // Cancel orders in batches of 50
    //const BATCH_SIZE: usize = 5;
    let total_orders = orders.len();

    for order in orders {
        //let a = market.get_instance().methods().cancel_order(order);
        let b = market.cancel_order(order).await;
        println!("result: {:?}", b);
    }
    /*
    if total_orders > 0 {
        for (batch_index, batch_orders) in orders.chunks(BATCH_SIZE).enumerate() {
            // Prepare multi_call_handler for this batch
            let mut multi_call_handler = CallHandler::new_multi_call(wallet.clone());

            for order_id in batch_orders {
                let cancel_order_call = market.get_instance().methods().cancel_order(*order_id);
                multi_call_handler = multi_call_handler.add_call(cancel_order_call);
            }

            println!(
                "Cancelling batch {} with {} orders",
                batch_index + 1,
                batch_orders.len()
            );

            // Execute the prepared calls for this batch
            let cancel_order_multicall_tx = multi_call_handler.submit().await?;
            println!("Calnceling responce: {:?}", cancel_order_multicall_tx);
            println!("Batch {} cancelled successfully", batch_index + 1);
        }
    } else {
        println!("No orders to cancel");
    }
    */

    Ok(())
}

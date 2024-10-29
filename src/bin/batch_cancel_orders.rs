use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, programs::calls::CallHandler,
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
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id = env::var("BTC_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    // Fetching user orders
    let orders = market.user_orders(wallet_id).await?.value;
    println!("User Orders: {:?}", orders);

    // Cancel orders in batches of 50
    const BATCH_SIZE: usize = 50;
    let total_orders = orders.len();

    if total_orders > 0 {
        for (batch_index, batch_orders) in orders.chunks(BATCH_SIZE).enumerate() {
            // Prepare multi_call_handler for this batch
            let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());

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
            let _cancel_order_multicall_tx = multi_call_handler.submit().await?;
            println!("Batch {} cancelled successfully", batch_index + 1);
        }
    } else {
        println!("No orders to cancel");
    }

    Ok(())
}

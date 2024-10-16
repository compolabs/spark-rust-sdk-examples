use dotenv::dotenv;
use std::env;

use fuels::{accounts::{provider::Provider, wallet::WalletUnlocked}, types::{Bits256, ContractId}};
use std::str::FromStr;

use anyhow::Result;
use spark_market_sdk::SparkMarketContract;

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id = env::var("BTC_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id).unwrap();
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Getting Fees from Spark Market
    //let order_id = "0xe887312d717eeb1672d86337584f47b76ae0847d1f9ceaf6da9a3f2beba608d5".to_owned();
    let order_id = "0xaa4dd886529b23bd5bdc123512e300bcb4b64cd203c4bbc759e3070e6f590d98".to_owned();
    //let order_id = "0xa6715f39df1b41728d03b11d0bfa8de94b4fec31d16e31decfadff7dca7a2dda".to_owned();
    //let order_id = "0xd032a66d3dbd62ce031d602ca18ac958e85f0d582684cf5ca96dd5497645f473".to_owned();
    //let order_id = "0x4616ee6c6e00b105f8fae203f2c3239b380f22232cd83a36c0d9d439235ef0d2".to_owned();
    let order_id_bits = Bits256::from_hex_str(&order_id).unwrap();
    let order_change_info = market.order_change_info(order_id_bits).await.unwrap();
    println!("====================");
    let order_status = market.order(order_id_bits).await.unwrap();
    println!("order_status {:?}", order_status.value);
    println!("====================");
    for a in order_change_info.value {

        println!("{:?}", a);
    }
    /*
    let matcher_fee = market.matcher_fee().await?.value;
    println!("matcher_fee: {:?}", matcher_fee);

    let protocol_fee = market.protocol_fee().await?.value;
    println!("protocol fee: {:?}", protocol_fee);
    */

    Ok(())
}

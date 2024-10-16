use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::AssetId,
    types::ContractId, types::Identity,
};
use std::str::FromStr;

use anyhow::Result;
use spark_market_sdk::{OrderType, SparkMarketContract};

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

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    let btc_id: String = env::var("BTC_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    // Getting asset balances
    let account = market.account(wallet_id).await?.value;
    let liquid_base = account.liquid.base;
    let liquid_quote = account.liquid.quote;

    println!("BTC Balance: {:?}", liquid_base);
    println!("USDC Balance: {:?}", liquid_quote);
    let order_buy = "0x25dbbbf85a97086eaaffde859dcf93d774d3c249bfd1281753d41ef161e1aba5";
    let order_sell = "0x7308df1f5e433fedfe275150dcb40befd317b1966bcf11276c7f21fb76a02605";
    let order_sell2 = "0x48b50d7bc0aed18c37edeb2c5cb47bfbd4d83d102b8c61d313be5ed097f2be0e";
    let orders = vec![order_buy,order_sell,order_sell2];

    let unique_bits256_ids: Vec<fuels::types::Bits256> = orders
        .iter()
        .map(|order| fuels::types::Bits256::from_hex_str(order).unwrap())
        .collect();
    println!("trying to match...");
    for o in orders {
        println!("{:?}", o);
    }
    let res = market.match_order_many(unique_bits256_ids).await;
    println!("--------");
    println!("{:?}",res);


    Ok(())
}

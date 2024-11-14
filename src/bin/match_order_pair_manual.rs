use dotenv::dotenv;
use std::env;

use anyhow::Result;
use fuels::types::Bits256;
use fuels::{accounts::provider::Provider, accounts::wallet::WalletUnlocked, types::ContractId};
use hex;
use spark_market_sdk::SparkMarketContract;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id = env::var("ETH_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::connect(provider_url).await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id).unwrap();
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    println!("Wallet Address: {:?}", main_wallet.address().to_string());

    // Paste your order IDs here
    let buy_order_id_str = "0xa3c5a22381727adc67d125aa01e24065516ad0c662e1258e85f5efdc9d45b641"; // Replace with your buy order ID
    let sell_order_id_str = "0x401809b4af32ed2ca5297efec1d67d8276265efbad264d80c4f2b63c1f4334db"; // Replace with your sell order ID

    // Convert order IDs to Bits256
    let buy_order_id = bits256_from_hex_str(buy_order_id_str)?;
    let sell_order_id = bits256_from_hex_str(sell_order_id_str)?;

    // Match the orders
    println!(
        "Matching Orders: Buy {:?} with Sell {:?}",
        buy_order_id, sell_order_id
    );
    market.match_order_pair(buy_order_id, sell_order_id).await?;
    println!("Orders Matched Successfully");

    Ok(())
}

fn bits256_from_hex_str(hex_str: &str) -> Result<Bits256> {
    let hex_str = if let Some(stripped) = hex_str.strip_prefix("0x") {
        stripped
    } else {
        hex_str
    };

    let mut bytes = [0u8; 32];
    hex::decode_to_slice(hex_str, &mut bytes)?;
    Ok(Bits256(bytes))
}

use dotenv::dotenv;
// use reqwest::Identity;
use std::env;

use anyhow::Result;
use fuels::types::Bits256;
use fuels::{
    accounts::provider::Provider,
    accounts::wallet::WalletUnlocked,
    types::{Address, ContractId, Identity},
};
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

    let user_address_str = "0xe800BDd249e488A0f1679DD800CE48553527AC43ef6532DAea72E795BB3609E4";
    let address = Address::from_str(user_address_str).expect("Invalid address string");
    let user_identity: Identity = Identity::Address(address);

    let orders = market.user_orders(user_identity).await.unwrap().value;
    println!("user orders: {:?}", orders);

    // Paste your order IDs here
    let order_id_str = "0xdc19e2b2eff449d059415014b7d1ba12935f9658f835784aeac79630e81df11c"; // Replace with your buy order ID

    // Convert order IDs to Bits256
    let order_id = bits256_from_hex_str(order_id_str)?;
    println!("buy order: {:?}", order_id);

    let order_info = market.order(*orders.first().unwrap()).await.unwrap();
    let order_info_1 = market.order(order_id).await.unwrap();

    println!("equal: {:?}", order_info.value == order_info_1.value);
    println!("order info: {:?}", order_info.value);

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

use dotenv::dotenv;
use fuels::{
    prelude::{Provider, WalletUnlocked},
    types::ContractId,
};

use spark_market_sdk::{AssetType, MarketContract, OrderType};
use std::{env, str::FromStr};

const BASE_SIZE: u64 = 1; //units
const BASE_PRICE: u64 = 69000; //units
const ORDER_TYPE: OrderType = OrderType::Sell; //units

#[tokio::main]
async fn main() {
    // print_title("Create Order");
    dotenv().ok();
    let provider = Provider::connect("testnet.fuel.network").await.unwrap();
    let secret = env::var("PRIVATE_KEY").unwrap();
    let wallet =
        WalletUnlocked::new_from_private_key(secret.parse().unwrap(), Some(provider.clone()));
    println!("wallet address = {:?}", wallet.address());
    // let token_contract = TokenContract::new(
    //     &ContractId::from_str("0x3141a3f11e3f784364d57860e3a4dcf9b73d42e23fd49038773cefb09c633348")
    //         .unwrap()
    //         .into(),
    //     wallet.clone(),
    // );

    let contract_id = env::var("CONTRACT_ID").unwrap();
    let market =
        MarketContract::new(ContractId::from_str(&contract_id).unwrap(), wallet.clone()).await;

    let (
        _,
        _, // base_asset_id,
        base_asset_decimals,
        _, //quote_asset_id,
        _, //quote_asset_decimals,
        price_decimals,
        _,
    ) = market.config().await.unwrap().value;

    let base_size = BASE_SIZE * 10u64.pow(base_asset_decimals);
    let base_price = BASE_PRICE * 10u64.pow(price_decimals);

    market
        .open_order(base_size, AssetType::Base, ORDER_TYPE, base_price)
        .await
        .unwrap();
}

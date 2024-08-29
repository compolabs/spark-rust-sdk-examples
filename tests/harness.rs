use dotenv::dotenv;
use fuels::prelude::{ContractId, Provider, WalletUnlocked};
use reqwest;
use serde_json::{json, Value};
use spark_orderbook_sdk::OrderbookContract;
use src20_sdk::token_utils::Asset;
use std::env;
use std::str::FromStr;
use fuels::accounts::ViewOnlyAccount;
use fuels::types::AssetId;
//todo не понятно какую версию fuels надо использовать, когда версия у меня и в сдк отличается
// начинаются проблемы, надо в releases пистаь версию fuels еще
// так-же сдк токена не описано

//todo нужно место откуда тянуть эти контракты

const TOKEN_CONTRACT_ID: &str = "0x0713334e61ed73ba9421a3a49891953f9ccb7353828566b569752a82a39803e8";
const ORDERBOOK_CONTRACT_ID: &str = "0x0911d52d95a71dd484690636fb81db8596f54ee18fe5eb7e33842025d1dd80de";
const BTC_USDC_MARKET_CONTRACT_ID: &str = "0x9bc5f33c9a1bec6461500cd85b3ba1d8f0094a865b6b9c4367631e4111d0305d";
const ETH_USDC_MARKET_CONTRACT_ID: &str = "0x2e9f781674f292d4db1ad150e7685e1f1ebad3c1ba403a64fff54b019ed70765";
const BTC_ASSET_ID: &str = "0x38e4ca985b22625fff93205e997bfc5cc8453a953da638ad297ca60a9f2600bc";
const USDC_ASSET_ID: &str = "0x336b7c06352a4b736ff6f688ba6885788b3df16e136e95310ade51aa32dc6f05";
trait ContractIdExt {
    fn to_contract_id(&self) -> ContractId;
    fn to_asset_id(&self) -> AssetId;
}

impl ContractIdExt for str {
    fn to_contract_id(&self) -> ContractId {
        ContractId::from_str(self).unwrap()
    }
    fn to_asset_id(&self) -> AssetId {
        AssetId::from_str(self).unwrap()
    }
}

//todo orderbook_contract должен отдавать список маркетов и асетов
#[tokio::test]
async fn get_markets() {
    let wallet = setup_wallet().await;
    let orderbook = OrderbookContract::new(ORDERBOOK_CONTRACT_ID.to_contract_id(), wallet).await;
    let markets = orderbook.markets(vec![(BTC_ASSET_ID.to_asset_id(), USDC_ASSET_ID.to_asset_id())]).await.unwrap();
    println!("Markets: {:#?}", markets);
}

//todo получать инстансы токенов при посощи маркетов
#[tokio::test]
async fn get_tokens() {}

//todo
#[tokio::test]
async fn get_account_balance() {
    let asset_id = "";
    let wallet = setup_wallet().await;
    let balance = wallet.get_asset_balance(&AssetId::from_str(asset_id).unwrap()).await.unwrap();
    println!("Balance {:?}", balance);
}

//fixme` Response errors; InsufficientMaxFee { max_fee_from_policies: 431, max_fee_from_gas_price: 326087 }`
#[tokio::test]
async fn get_market_balance() {
    let asset_id = "";
    let wallet = setup_wallet().await;
    let balance = wallet.get_asset_balance(&AssetId::from_str(asset_id).unwrap()).await.unwrap();
    println!("Balance {:?}", balance);
}

#[tokio::test]
async fn get_orders() {
    //     let client = reqwest::Client::new();
    //     let query = r#"
    //     query MyQuery {
    //       ActiveSellOrder(order_by: {price: asc}, limit: 2) {
    //         amount
    //         id
    //         price
    //       }
    //       ActiveBuyOrder(order_by: {price: desc}, limit: 5) {
    //         amount
    //         id
    //         price
    //       }
    //     }
    //     "#;
    //
    //     let res = client.post("http://localhost:8080/v1/graphql")
    //         .json(&json!({
    //             "query": query,
    //             "variables": {}
    //         }))
    //         .send()
    //         .await
    //         .expect("Failed to send request");
    //
    //     assert!(res.status().is_success());
    //
    //     let body: Value = res.json().await.expect("Failed to parse JSON");
    //
    //     // Проверяем, что в ответе есть данные
    //     assert!(body["data"].is_object());
    //
    //     // Проверяем, что есть ActiveSellOrder и ActiveBuyOrder
    //     assert!(body["data"]["ActiveSellOrder"].is_array());
    //     assert!(body["data"]["ActiveBuyOrder"].is_array());
    //
    //     // Проверяем, что ActiveSellOrder содержит не более 2 элементов
    //     assert!(body["data"]["ActiveSellOrder"].as_array().unwrap().len() <= 2);
    //
    //     // Проверяем, что ActiveBuyOrder содержит не более 5 элементов
    //     assert!(body["data"]["ActiveBuyOrder"].as_array().unwrap().len() <= 5);
    //
    //     // Проверяем структуру данных для каждого ордера
    //     for order in body["data"]["ActiveSellOrder"].as_array().unwrap() {
    //         assert!(order["amount"].is_string());
    //         assert!(order["id"].is_string());
    //         assert!(order["price"].is_string());
    //     }
    //
    //     for order in body["data"]["ActiveBuyOrder"].as_array().unwrap() {
    //         assert!(order["amount"].is_string());
    //         assert!(order["id"].is_string());
    //         assert!(order["price"].is_string());
    //     }
    //
    //     println!("Response: {:#?}", body);
}

#[tokio::test]
async fn get_trades() {}

#[tokio::test]
async fn mint() {}

#[tokio::test]
async fn deposit() {}

#[tokio::test]
async fn withdraw() {}

#[tokio::test]
async fn open_order_limit() {}

#[tokio::test]
async fn fulfill_order() {}

#[tokio::test]
async fn close_order() {}

#[tokio::test]
async fn multicall() {}


async fn setup_wallet() -> WalletUnlocked {
    dotenv().ok();
    let provider = Provider::connect("testnet.fuel.network").await.unwrap();
    let secret = env::var("PRIVATE_KEY").unwrap();
    WalletUnlocked::new_from_private_key(secret.parse().unwrap(), Some(provider.clone()))
}


pub(crate) struct Assets {
    pub(crate) base: Asset,
    pub(crate) quote: Asset,
    // pub(crate) random: Asset,
    // pub(crate) fuel: Asset,
}

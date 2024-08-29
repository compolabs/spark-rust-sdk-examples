use dotenv::dotenv;
use fuels::{
    accounts::ViewOnlyAccount,
    prelude::{Provider, WalletUnlocked},
    types::{Address, AssetId, Bits256, ContractId, Identity},
};

use rand::Rng;
use spark_market_sdk::{AssetType, MarketContract, OrderType};
use src20_sdk::token_utils::{Asset, TokenContract};
use std::{env, str::FromStr, thread::sleep, time::Duration};
struct OrderConfig {
    pub amount: u64,
    pub order_type: OrderType,
    pub price: u64,
}

pub(crate) struct User {
    pub(crate) wallet: WalletUnlocked,
}

impl User {
    pub(crate) fn address(&self) -> Address {
        Address::from(self.wallet.address())
    }

    pub(crate) fn identity(&self) -> Identity {
        Identity::Address(self.address())
    }

    pub(crate) async fn balance(&self, asset: &AssetId) -> u64 {
        self.wallet.get_asset_balance(asset).await.unwrap()
    }
}

pub(crate) struct Assets {
    pub(crate) base: Asset,
    pub(crate) quote: Asset,
    // pub(crate) random: Asset,
    // pub(crate) fuel: Asset,
}

const TOKEN_CONTRACT_ID: &str =
    "0x3141a3f11e3f784364d57860e3a4dcf9b73d42e23fd49038773cefb09c633348";

const BASE_ASSET: &str = "BTC";
const QUOTE_ASSET: &str = "USDC";

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    // dotenv().ok();
    // let provider = Provider::connect("testnet.fuel.network").await.unwrap();
    // let secret = env::var("PRIVATE_KEY").unwrap();
    // let wallet =
    //     WalletUnlocked::new_from_private_key(secret.parse().unwrap(), Some(provider.clone()));
    //
    // let token_contarct = TokenContract::new(
    //     &ContractId::from_str(TOKEN_CONTRACT_ID).unwrap().into(),
    //     wallet.clone(),
    // );
    //
    // let contract_id = env::var("CONTRACT_ID").unwrap();
    // let contract =
    //     MarketContract::new(ContractId::from_str(&contract_id).unwrap(), wallet.clone()).await;
    //
    // let user0 = User {
    //     wallet: wallet.clone(),
    // };
    // let user1 = User {
    //     wallet: wallet.clone(),
    // };
    // let assets = Assets {
    //     base: Asset::new(
    //         wallet.clone(),
    //         token_contarct.contract_id().into(),
    //         BASE_ASSET,
    //     ),
    //     quote: Asset::new(
    //         wallet.clone(),
    //         token_contarct.contract_id().into(),
    //         QUOTE_ASSET,
    //     ),
    // };
    // while (true) {
    //     for _ in 0..25 {
    //         // Specify the range for order amounts and prices
    //         let amount_range = 100_000..10_000_000; // 0.001 BTC to 0.1 BTC
    //         let price_range = 50_000_000_000_000_i64..70_000_000_000_000_i64; // 1 USDC to 200k USDC
    //         let price_variation_range = -500..=500; // Range for price variation
    //
    //         let mut rng = rand::thread_rng();
    //         let mut order_configs: Vec<OrderConfig> = Vec::new();
    //
    //         let base_price = rng.gen_range(price_range.clone());
    //
    //         for _ in 0..3 {
    //             // Generate a random variation within the range of -500 to 500
    //             let buy_price_variation: i64 = rng.gen_range(price_variation_range.clone()) + 500;
    //             let sell_price_variation: i64 = rng.gen_range(price_variation_range.clone()) - 500;
    //
    //             // Adjust the buy and sell order prices by their respective variations
    //             let buy_order_price = (base_price as i64 + buy_price_variation).max(0) as u64;
    //             let sell_order_price = (base_price as i64 + sell_price_variation).max(0) as u64;
    //
    //             let buy_order = OrderConfig {
    //                 amount: rng.gen_range(amount_range.clone()),
    //                 order_type: OrderType::Buy,
    //                 price: buy_order_price,
    //             };
    //             let sell_order = OrderConfig {
    //                 amount: rng.gen_range(amount_range.clone()),
    //                 order_type: OrderType::Sell,
    //                 price: sell_order_price,
    //             };
    //             order_configs.push(buy_order);
    //             order_configs.push(sell_order);
    //         }
    //
    //         // println!("order_configs = {:#?}", order_configs);
    //
    //         let base_deposit = 1_000_000_000_u64; // 10 BTC
    //         let quote_deposit = 1_000_000_000_000_u64; // 1m USDC
    //
    //         // user0 deposits and opens 6 orders
    //         assets
    //             .base
    //             .mint(user0.wallet.address().into(), base_deposit)
    //             .await
    //             .unwrap();
    //         contract
    //             .with_account(&user0.wallet)
    //             .await?
    //             .deposit(base_deposit, assets.base.asset_id)
    //             .await?;
    //         assets
    //             .quote
    //             .mint(user0.wallet.address().into(), quote_deposit)
    //             .await
    //             .unwrap();
    //         contract
    //             .with_account(&user0.wallet)
    //             .await?
    //             .deposit(quote_deposit, assets.quote.asset_id)
    //             .await?;
    //
    //         let mut order_ids: Vec<Bits256> = Vec::new();
    //         for config in &order_configs {
    //             order_ids.push(
    //                 contract
    //                     .with_account(&user0.wallet)
    //                     .await?
    //                     .open_order(
    //                         config.amount,
    //                         AssetType::Base,
    //                         config.order_type.clone(),
    //                         config.price,
    //                     )
    //                     .await?
    //                     .value,
    //             );
    //         }
    //
    //         // user1 deposits and opens 6 orders
    //         assets
    //             .base
    //             .mint(user1.wallet.address().into(), base_deposit)
    //             .await
    //             .unwrap();
    //         contract
    //             .with_account(&user1.wallet)
    //             .await?
    //             .deposit(base_deposit, assets.base.asset_id)
    //             .await?;
    //         assets
    //             .quote
    //             .mint(user1.wallet.address().into(), quote_deposit)
    //             .await
    //             .unwrap();
    //         contract
    //             .with_account(&user1.wallet)
    //             .await?
    //             .deposit(quote_deposit, assets.quote.asset_id)
    //             .await?;
    //
    //         for config in &order_configs {
    //             order_ids.push(
    //                 contract
    //                     .with_account(&user1.wallet)
    //                     .await?
    //                     .open_order(
    //                         config.amount,
    //                         AssetType::Base,
    //                         config.order_type.clone(),
    //                         config.price,
    //                     )
    //                     .await?
    //                     .value,
    //             );
    //         }
    //
    //         sleep(Duration::from_secs(1));
    //     }
    // }

    Ok(())
}
//get_markets
//get_tokens
//get_account_balance
//get_market_balance
//get_orders
//get_trades
//mint
//deposit
//withdraw
//open_order limit
//fulfill_order
//close_order
//multicall

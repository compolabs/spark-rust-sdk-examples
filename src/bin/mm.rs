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
    dotenv().ok();
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
    // let market_contract_id = env::var("CONTRACT_ID").unwrap();
    // let market =
    //     MarketContract::new(ContractId::from_str(&market_contract_id).unwrap(), wallet.clone()).await;

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
    //
    // //----
    // let base_balance = wallet.get_asset_balance(&assets.base.asset_id).await.unwrap();
    // let quote_balance = wallet.get_asset_balance(&assets.quote.asset_id).await.unwrap();
    // let deposit = market.account(wallet.address().into()).await.unwrap().value.unwrap().liquid;

    // println!("base_balance {:#?}", base_balance);
    // println!("quote_balance {:#?}", quote_balance);
    // println!("deposit {:#?}", deposit);


    //----

    Ok(())
}

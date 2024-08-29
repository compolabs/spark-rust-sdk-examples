use dotenv::dotenv;
use fuels::prelude::{Provider, WalletUnlocked};

use std::env;

#[tokio::main]
async fn main() {
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

    //todo
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

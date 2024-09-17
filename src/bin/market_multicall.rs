use dotenv::dotenv;
use std::{env, error::Error, str::FromStr};

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    prelude::CallParameters,
    programs::calls::CallHandler,
    types::{AssetId, ContractId, Identity},
};

use spark_market_sdk::{AssetType, OrderType, SparkMarketContract};

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

// 1) Run withdraw.rs on BTC & ETH markets
// 2) Run batch_open_orders.rs for BTC market
// 3) Run this script:

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let btc_usdc_contract_id = env::var("BTC_USDC_CONTRACT_ID")?;
    let eth_usdc_contract_id = env::var("ETH_USDC_CONTRACT_ID")?;

    // Connect to provider
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();

    let btc_contract_id = ContractId::from_str(&btc_usdc_contract_id)?;
    let btc_market = SparkMarketContract::new(btc_contract_id.clone(), main_wallet.clone()).await;

    let eth_contract_id = ContractId::from_str(&eth_usdc_contract_id)?;
    let eth_market = SparkMarketContract::new(eth_contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("Wallet Address: {:?}", main_wallet.address().to_string());

    let usdc_id: String = env::var("USDC_ID")?;

    // Retrieve account balances in BTC/USDC market
    let btc_account = btc_market.account(wallet_id.clone()).await?.value;
    println!("BTC account before: {:?}", btc_account);

    // Retrieve account balances in ETH/USDC market
    let eth_account = eth_market.account(wallet_id.clone()).await?.value;
    println!("ETH account before: {:?}", eth_account);

    // Calculate total withdrawable amounts from BTC market
    let base_withdraw_amount = btc_account.liquid.base + btc_account.locked.base;
    let quote_withdraw_amount = btc_account.liquid.quote + btc_account.locked.quote;

    // Start Multi-call: Cancel orders and withdraw from BTC market, deposit and open order in ETH market
    let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone());

    // Retrieve user orders in BTC/USDC market
    let orders = btc_market.user_orders(wallet_id.clone()).await?.value;

    // Cancel open orders in BTC/USDC market
    for order in orders {
        let cancel_order_call = btc_market
            .get_instance()
            .methods()
            .cancel_order(order)
            .call_params(CallParameters::default())
            .unwrap();

        multi_call_handler = multi_call_handler.add_call(cancel_order_call);
    }

    // Withdraw base asset (e.g., BTC) if balance is greater than zero
    if base_withdraw_amount > 0 {
        let withdraw_base_call = btc_market
            .get_instance()
            .methods()
            .withdraw(base_withdraw_amount, AssetType::Base)
            .call_params(CallParameters::new(0, AssetId::default(), 100_000_000)) // Adjust gas limit
            .unwrap();

        multi_call_handler = multi_call_handler.add_call(withdraw_base_call);
    }

    // Withdraw quote asset (e.g., USDC) if balance is greater than zero
    if quote_withdraw_amount > 0 {
        let withdraw_quote_call = btc_market
            .get_instance()
            .methods()
            .withdraw(quote_withdraw_amount, AssetType::Quote)
            .call_params(CallParameters::new(0, AssetId::default(), 100_000_000)) // Adjust gas limit
            .unwrap();

        multi_call_handler = multi_call_handler.add_call(withdraw_quote_call);
    }

    // Depositing USDC to ETH/USDC market
    let usdc_asset_id = AssetId::from_str(&usdc_id)?;
    let usdc_amount = format_value_with_decimals(3_000, 6);

    let deposit_usdc_call = eth_market
        .get_instance()
        .methods()
        .deposit()
        .call_params(CallParameters::new(usdc_amount, usdc_asset_id, 100_000_000)) // Adjust gas limit
        .unwrap();

    multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);

    // multi_call_handler = multi_call_handler.add_call(buy_open_order_call);

    // Execute the multi-call
    let _multicall_tx_result = multi_call_handler.submit().await;

    println!("is ok: {:?}", _multicall_tx_result.is_ok());
    // Output the result of the multi-call
    // println!("Multicall result: {:?}", multicall_tx_result);

    // Retrieve account balances in BTC/USDC market after multi-call
    let btc_account_after = btc_market.account(wallet_id.clone()).await?.value;
    println!("BTC account after: {:?}", btc_account_after);

    // Retrieve account balances in ETH/USDC market after multi-call
    let eth_account_after = eth_market.account(wallet_id.clone()).await?.value;
    println!("ETH account after: {:?}", eth_account_after);

    Ok(())
}

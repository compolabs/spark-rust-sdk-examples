use dotenv::dotenv;
use std::{env, error::Error, str::FromStr};

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    prelude::{CallParameters, VariableOutputPolicy},
    programs::calls::CallHandler,
    types::transaction::TxPolicies,
    types::{AssetId, ContractId, Identity},
};

use spark_market_sdk::{AssetType, SparkMarketContract};

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
    let base_withdraw_amount = btc_account.liquid.base;
    let quote_withdraw_amount = btc_account.liquid.quote;

    // Retrieve user orders in BTC/USDC market
    let orders = btc_market.user_orders(wallet_id.clone()).await?.value;

    // Start Multi-call: Cancel orders and withdraw from BTC market, deposit and open order in ETH market
    let mut multi_call_handler = CallHandler::new_multi_call(main_wallet.clone())
        .with_variable_output_policy(VariableOutputPolicy::Exactly(2));

    if orders.len() > 0 {
        // Cancel open orders in BTC/USDC market
        for order_id in orders.clone() {
            let cancel_order_call = btc_market.get_instance().methods().cancel_order(order_id);
            multi_call_handler = multi_call_handler.add_call(cancel_order_call);
        }
    }

    // Withdraw base asset (e.g., BTC) if balance is greater than zero
    if base_withdraw_amount > 0 {
        println!("base amount: {:?}", base_withdraw_amount);
        let withdraw_base_call = btc_market
            .get_instance()
            .methods()
            .withdraw(base_withdraw_amount, AssetType::Base);

        multi_call_handler = multi_call_handler.add_call(withdraw_base_call);
    }

    // Withdraw quote asset (e.g., USDC) if balance is greater than zero
    if quote_withdraw_amount > 0 {
        let withdraw_quote_call = btc_market
            .get_instance()
            .methods()
            .withdraw(quote_withdraw_amount, AssetType::Quote);

        multi_call_handler = multi_call_handler.add_call(withdraw_quote_call);
    }

    // Depositing USDC to ETH/USDC market
    let usdc_asset_id = AssetId::from_str(&usdc_id)?;
    let deposit_usdc_call_params =
        CallParameters::new(quote_withdraw_amount, usdc_asset_id, 20_000_000);
    let deposit_usdc_call = eth_market
        .get_instance()
        .methods()
        .deposit()
        .call_params(deposit_usdc_call_params)
        .unwrap();

    multi_call_handler = multi_call_handler.add_call(deposit_usdc_call);

    // Execute the multi-call
    let multicall_tx_result = multi_call_handler.submit().await?;

    let tx_id = multicall_tx_result.tx_id(); // Save the tx_id before moving `multicall_tx_result`
    println!("multicall transaction id: 0x{:?}", tx_id);

    // Retrieve account balances in BTC/USDC market after multi-call
    let btc_account_after = btc_market.account(wallet_id.clone()).await?.value;
    println!("BTC account after: {:?}", btc_account_after);

    // Retrieve account balances in ETH/USDC market after multi-call
    let eth_account_after = eth_market.account(wallet_id.clone()).await?.value;
    println!("ETH account after: {:?}", eth_account_after);

    Ok(())
}

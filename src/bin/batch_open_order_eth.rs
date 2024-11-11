use dotenv::dotenv;
use std::{env, error::Error, str::FromStr};

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked},
    prelude::CallParameters,
    programs::calls::CallHandler,
    types::{AssetId, ContractId, Identity},
};

use spark_market_sdk::{OrderType, SparkMarketContract};

use tokio::time::{sleep, Duration};

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readable_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

// Might fail if you don't have enough testnet ETH
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let mnemonic = env::var("MNEMONIC")?;
    let contract_id_str = env::var("ETH_USDC_CONTRACT_ID")?;
    let implementation_contract_id_str = env::var("ETH_USDC_IMPLEMENTATION")?;

    // Connect to provider
    let provider_url = env::var("PROVIDER")?;
    let provider = Provider::connect(provider_url).await?;

    let main_wallet =
        WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone())).unwrap();
    let contract_id = ContractId::from_str(&contract_id_str)?;
    let implementation_contract_id = ContractId::from_str(&implementation_contract_id_str)?;
    let market = SparkMarketContract::new(contract_id.clone(), main_wallet.clone()).await;

    // Fuel wallet address
    let wallet_id: Identity = main_wallet.address().into();
    println!("wallet {:?}", main_wallet.address().to_string());

    let eth_id: String = env::var("ETH_ID")?;
    let usdc_id: String = env::var("USDC_ID")?;

    let eth_id = AssetId::from_str(&eth_id)?;
    let usdc_id = AssetId::from_str(&usdc_id)?;

    // Amounts to be used
    let eth_amount = format_value_with_decimals(1, 7);
    let usdc_amount = format_value_with_decimals(50, 6);

    // Get user's current balances in the contract
    let account = market.account(wallet_id.clone()).await?.value;
    let liquid_base = account.liquid.base;
    let liquid_quote = account.liquid.quote;

    println!("Current base balance: {}", liquid_base);
    println!("Current quote balance: {}", liquid_quote);

    // If the user's balance is less than the required amount, deposit the difference
    if liquid_base < eth_amount {
        println!("ETH deposit");
        let eth_deposit_amount = eth_amount - liquid_base;

        let deposit_eth_call_params = CallParameters::new(eth_deposit_amount, eth_id, 1_000_000);
        let deposit_eth_call = market
            .get_instance()
            .methods()
            .deposit()
            .with_contract_ids(&[contract_id.into(), implementation_contract_id.into()])
            .call_params(deposit_eth_call_params)
            .unwrap();

        // Execute the deposit call
        let _deposit_eth_result = deposit_eth_call.call().await?;
        println!("Deposited {} base asset", eth_deposit_amount);
    } else {
        println!("Sufficient base balance, no deposit needed");
    }

    if liquid_quote < usdc_amount {
        println!("USDC deposit");
        let usdc_deposit_amount = usdc_amount - liquid_quote;

        let deposit_usdc_call_params = CallParameters::new(usdc_deposit_amount, usdc_id, 1_000_000);
        let deposit_usdc_call = market
            .get_instance()
            .methods()
            .deposit()
            .with_contract_ids(&[contract_id.into(), implementation_contract_id.into()])
            .call_params(deposit_usdc_call_params)
            .unwrap();

        // Execute the deposit call
        let _deposit_usdc_result = deposit_usdc_call.call().await?;
        println!("Deposited {} quote asset", usdc_deposit_amount);
    } else {
        println!("Sufficient quote balance, no deposit needed");
    }

    // Now proceed to create the open order calls
    let mut multi_call_handler: CallHandler<
        WalletUnlocked,
        Vec<fuels::programs::calls::ContractCall>,
        (),
    > = CallHandler::new_multi_call(main_wallet.clone());

    // Creating Buy / Sell Limit Orders in a single transaction
    let protocol_fee = market.protocol_fee().await?.value;
    println!("protocol_fee: {:?}", protocol_fee);

    let open_order_call_params = CallParameters::default().with_gas_forwarded(20_000_000); // Increased gas

    let buy_order_type = OrderType::Buy;
    let buy_order_amount = 100_000; // 0.0001 ETH
    let buy_start_price = 3_000u64;
    let sell_order_amount = 100_000;
    let sell_start_price = 3_001u64;
    let step = 1;

    for i in 0..1 {
        let buy_open_price = (buy_start_price + i * step) * 1_000_000_000_u64;
        let sell_open_price = (sell_start_price + i * step) * 1_000_000_000_u64;

        let buy_open_order_call = market
            .get_instance()
            .methods()
            .open_order(buy_order_amount, buy_order_type.clone(), buy_open_price)
            .call_params(open_order_call_params.clone())
            .unwrap();

        let sell_open_order_call = market
            .get_instance()
            .methods()
            .open_order(sell_order_amount, OrderType::Sell, sell_open_price)
            .call_params(open_order_call_params.clone())
            .unwrap();

        multi_call_handler = multi_call_handler.add_call(buy_open_order_call);
        multi_call_handler = multi_call_handler.add_call(sell_open_order_call);
    }

    // Include contracts when submitting the multicall
    let multicall_tx_result = multi_call_handler.submit().await?;
    println!(
        "Submitted open orders in a multicall transaction: 0x{:?}",
        multicall_tx_result.tx_id().to_string()
    );

    sleep(Duration::from_secs(5)).await;

    let orders = market.user_orders(wallet_id).await?.value;
    println!("Number of Orders: {:?}", orders.len());

    Ok(())
}

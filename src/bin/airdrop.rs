use dotenv::dotenv;
use std::env;

use fuels::{
    accounts::{provider::Provider, wallet::WalletUnlocked, Account},
    crypto::SecretKey,
    types::{bech32::Bech32Address, transaction::TxPolicies, Address},
};
use std::str::FromStr;

use std::error::Error;

const ASSET_ID: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const AMOUNT: u64 = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // Environment variables
    let private_key = env::var("PRIVATE_KEY")?;
    let provider = Provider::connect("testnet.fuel.network").await?;

    let main_wallet = WalletUnlocked::new_from_private_key(
        SecretKey::from_str(&private_key).unwrap(),
        Some(provider.clone()),
    );
    println!("Main wallet: {:?}", main_wallet.address());
    let addresses = read_addresses_from_file("addresses")?;

    println!(
        "Starting airdrop of {} tokens to {} addresses",
        AMOUNT,
        addresses.len()
    );

    for address in addresses {
        match main_wallet
            .transfer(
                &Bech32Address::from(Address::from_str(&address).unwrap()),
                AMOUNT,
                ASSET_ID.parse().unwrap(),
                TxPolicies::default(),
            )
            .await
        {
            Ok(_) => {
                println!("Successfully sent {} tokens to {}", AMOUNT, address);
            }
            Err(e) => {
                println!("Failed to send tokens to {}: {}", address, e);
            }
        }
    }

    println!("Airdrop completed!");

    Ok(())
}

//Файл должен содержать адреса Fuel кошельков, по одному адресу на строку. Пустые строки игнорируются.
pub fn read_addresses_from_file(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let addresses: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(addresses)
}

pub fn format_value_with_decimals(value: u64, decimals: u32) -> u64 {
    value * 10u64.pow(decimals)
}

pub fn format_to_readble_value(value: u64, decimals: u32) -> f64 {
    value as f64 / 10u64.pow(decimals) as f64
}

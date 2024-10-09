# Spark Rust SDK Examples

This repository contains examples of how to deposit liquidity, open orders, batch cancel orders, and withdraw liquidity from Spark. 

On using fuel's multicall functionality it is possible to open and close multiple orders in a single transaction.


## Deployment Addresses

Spark Registry
0x194987ad2314d2de50646078ac1841f00b2dffda863a7d3dd421d220eb83d019

BTC/USDC Spark Market
0x7b88385ae73dd3ccc62012e7a52cddd05c7e82ad54a5df721dfa0c1f8b5998f0

ETH/USDC Spark Market
0xc18094a283193c9b4726d2f644ed07ec9806bbe60a0688d45bffb26c379c1428

## Default Fees

### Protocol Fees:
| maker | taker | Volume of quote asset |
| --- | --- | --- |
| 0.1% | 0.15% | 10000_000000 |
| 0.08% | 0.13% | 50000_000000 |
| 0.06% | 0.09% | 100000_000000 |
| 0.02% | 0.07% | 500000_000000 |
| 0.01% | 0.05% | 1000000_000000 |

*On the contract side, the protocol fee units are in base 1e4 (10_000)*

### Matcher Fee: 
$0.001 USD
*On the contract side, the matcher fee is in the base unit of the quote asset. If the quote asset is USDC, this will be in base 1e6*


## Minting testnet tokens:

To mint testnet tokens via the UI: [Spark Testnet Faucet](https://app.sprk.fi/#/faucet)


# Running tests:
```
rustup target add wasm32-unknown-unknown
```

```
cargo build
```

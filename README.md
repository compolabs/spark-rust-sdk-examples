# Spark Rust SDK Examples

This repository contains examples of how to deposit liquidity, open orders, batch cancel orders, and withdraw liquidity from Spark. 

On using fuel's multicall functionality it is possible to open and close multiple orders in a single transaction.


## Deployment Addresses

Spark Market Registry: `0xfc3439e66c15c0b8e8e02e0fe4951eecd6001639a6aa8a650885af62ea92540f`

Spark Market BTC/USDC: `0xbc69e42348c9a992100970e8469247b58a517c2f7dd373741c6707cda35ccfc6`

Spark Market ETH/USDC: `0x10860c51de3a37c47ebff2e08a810d9e2963ef161ee3a88274520938c85914e8`

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
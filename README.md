# Spark Rust SDK Examples

This repository contains examples of how to deposit liquidity, open orders, batch cancel orders, and withdraw liquidity from Spark. 

On using fuel's multicall functionality it is possible to open and close multiple orders in a single transaction.


## Deployment Addresses

Spark Market Registry: `0xe943c485046d9e68e5fcb724a508f2af7f65141637c0aabcd620597437881225`

Spark Market BTC/USDC: `0x30bd67d27a021ae7acc982fdcbf905d3ea229f914e30b70860c0577457c87b19`

Spark Market ETH/USDC: `0xd2678aaafa555b09d974273c6ae308aa3be3f89a8585a68bc05aa74e04b641df`

Spark Market ETH/BTC: `0x95fad7c38e35b795eb993758a891710b7ce18958f0d6a4451bd0be70401fc62e`

## Default Fees

### Protocol Fees:
| maker | taker | Volume of base asset |
| --- | --- | --- |
| 0.1% | 0.15% | 100 |
| 0.08% | 0.13% | 500 |
| 0.06% | 0.09% | 1000 |
| 0.02% | 0.07% | 5000 |
| 0.01% | 0.05% | 10_000 |

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
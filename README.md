# Spark Rust SDK Examples

This repository contains examples of how to deposit liquidity, open orders, batch cancel orders, and withdraw liquidity from Spark. 

On using fuel's multicall functionality it is possible to open and close multiple orders in a single transaction.



## Deployment Addresses

Spark Market Registry: `0x8f7935292f3da69aec797926029c864d7ec6d03c72f7347b4fd517ba4a7b78fb`

Spark Market BTC/USDC: `0x416ccdaf69881ae345537b1844d1511b4103379fca43b8c2190aae8b42f08173`

Spark Market ETH/USDC: `0x7d1da52a221897ebc88dc6a5d4623e704f7d64022d498b0438827dd79b6e5457`

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
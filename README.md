# Spark Rust SDK Examples

This repository contains examples of how to deposit liquidity, open orders, batch cancel orders, and withdraw liquidity from Spark. 

On using fuel's multicall functionality it is possible to open and close multiple orders in a single transaction.


## Minting testnet tokens:

To mint testnet tokens via the UI: [Spark Testnet Faucet](https://app.sprk.fi/#/faucet)

# Default Fee Schedule for all deployed test contracts:

| maker  | taker  | Volume of base asset |
|--------|--------|----------------------|
| 0.1%   | 0.15%  | 100                  |
| 0.08%  | 0.13%  | 500                  |
| 0.06%  | 0.09%  | 1000                 |
| 0.02%  | 0.07%  | 5000                 |
| 0.01%  | 0.05%  | 10_000               |

On the contract side, the protocol fee units are in base 1e4 (10,000). 

**Default Matcher Fee: $0.001 USD**

On the contract side, the matcher fee is in the base unit of the quote asset. If the quote asset is USDC, this will be in base 1e6.

Both matcher & protocol fees are taken in the quote asset, i.e. USDC, USDT, etc.

# *Deployments:*

## **Using fuel-rs 0.66.3**

### Spark Market BTC/USDC: `0xa08b87ce9c98a47511c78a3426b6b392e336b811aa29b9b2f6074405e3808c83`

Matcher Fee: 1000 ($0.001)

Protocol Fee schedule: `["10,15,0", "8,13,10000000000", "6,11,50000000000", "4,9,100000000000", "2,7,500000000000", "1,5,1000000000000"]`

### Spark Market ETH/USDC: `0xab74b4bf2fb3f699e20c1312b29d24b5368bac0e8b04d0c732a674a91854c6e8`

Matcher Fee: 1000 ($0.001)

Protocol Fee schedule: `["10,15,0", "8,13,10000000000", "6,11,50000000000", "4,9,100000000000", "2,7,500000000000", "1,5,1000000000000"]`


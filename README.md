# Spark Rust SDK Examples

This repository contains examples of how to deposit liquidity, open orders, batch cancel orders, and withdraw liquidity from Spark. 

On using fuel's multicall functionality it is possible to open and close multiple orders in a single transaction.



## Deployment Addresses

Spark Market Registry (vo.3.1): `0x0ced80a8ce2cc8a3d39a88483edd961b2b7e9e5028f27cd2766f03fc61c406c9`

Spark Market BTC/USDC (v0.3.1): `0x8c84df7be0c095c5bac97e66e24aa00f4f51d50e207f2687128bd180a804cff6`

Spark Market ETH/USDC (v0.3.1): `0x352f7acf2286f4bddc278cfe8b0f84313ba46a1b71b7b9e7c5fc4869c93db8bb`

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

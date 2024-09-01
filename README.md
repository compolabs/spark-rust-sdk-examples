# Spark Rust SDK Examples

This repository contains examples of how to deposit liquidity, open orders, batch cancel orders, and withdraw liquidity from Spark. 

On using fuel's multicall functionality it is possible to open and close multiple orders in a single transaction.


## Minting tokens:

To mint testnet tokens via the UI: [Spark Testnet Faucet](https://app.sprk.fi/#/faucet)

To mint testnet tokens via the CLI:

Clone the [multiasset-contract](https://github.com/compolabs/multiasset-contract), build, and run the following command inside the multiasset directory:
```
./target/release/multiasset_sdk core mint \
--recipient-id {recipeint address} \
--recipient-type address \
--asset {asset id} \ 
--amount 200000000 \
--contract-id 0xdc527289bdef8ec452f350c9b2d36d464a9ebed88eb389615e512a78e26e3509 \
--rpc "testnet.fuel.network" \
```

Example minting tokens: 
```
./target/release/multiasset_sdk core mint \
    --recipient-id 0xd1ebb551a2d58f024875bcc6798e4e2f8c8feec9da718a9f29d362f14531a3ef \
    --recipient-type address \
    --asset 0x38e4ca985b22625fff93205e997bfc5cc8453a953da638ad297ca60a9f2600bc \
    --amount 1000000000000 \
    --contract-id 0xdc527289bdef8ec452f350c9b2d36d464a9ebed88eb389615e512a78e26e3509 \
    --rpc "testnet.fuel.network"
```

# Ecodata Payout Contract

This repo contains an example CosmWasm smart contract for use in developing oracles for Phase 5 of Regen Network's Kontraua Test Net. More information on the Phase 5 challenge "Payout Contract for ecological credits" can be found [here](https://github.com/regen-network/testnets/blob/master/kontraua/challenges/phase-5/README.md).

## Build & Test

Run the following to build the contract from source, and run both unit & integration tests. This will test both the native rust code (unit tests), as well as the functionality of the compiled wasm code.

```rust
cargo build
cargo build --release --target wasm32-unknown-unknown
cargo test
```

## Uploading to Kontraua

For uploading the code to the Kontraua testnet, you can follow the instructions from the Phase 2 challenge [here](https://github.com/regen-network/testnets/blob/master/kontraua/challenges/phase-2/instructions.md#step---1--upload-your-contract-optional).

## Initializing the Contract

Initialization of the contract will most easily be done via `xrncli` as described in the Phase 2 instructions. The content of the InitMsg JSON should match the JSON Schema as illustrated in [./schema/init_msg.json](./schema/init_msg.json).

## Oracle Communication

The easiest way have an external application send transactions to a live CosmWasm chain is via the cosmwasm-js library. The most relevant documentation pages on cosmwasm-js are probably the [cosmwasm-js CLI readme](https://github.com/CosmWasm/cosmwasm-js/blob/master/packages/cli/README.md), and the tutorial for interacting with the [Mask contract](https://github.com/CosmWasm/cosmwasm-js/blob/master/packages/cli/MASK.md).


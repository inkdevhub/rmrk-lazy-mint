# RMRK Lazy Minting Proxy Contract

This is a proxy contract for RMRK to enable lazy minting.

### Purpose
RMRK is designed in a way that all tokens in collection needs to be pre-minted and equipped with assets to be useful, which increases costs for a collection creators.
This contract enables lazy token minting on RMRK contract. The proxy contract has single method `mint` which does the following:
  - mints RMRK token
  - adds random asset to the token
  - transfers the token to the method caller

To be able to use this contract some prerequisites (see e2e test for details) must be met:
- RMRK and catalog contracts are deployed
- parts added to the catalog contract (`catalog::addPartList`)
- asset entries added to the RMRK contract (call `multiAsset::addAssetEntry` for each entry you want to add)

### License
Apache 2.0

### 🏗️ How to use - Contracts
##### 💫 Build
- Use this [instructions](https://use.ink/getting-started/setup) to setup your ink!/Rust environment

```sh
cd rmrk_proxy
cargo contract build --release
```

##### 💫 Run unit and integration tests

```sh
cd rmrk_proxy
cargo test --features e2e-tests -- --nocapture
```
##### 💫 Deploy
First start your local node. Recommended is the latest [swanky-node](https://github.com/AstarNetwork/swanky-node/releases)
```sh
./target/release/swanky-node --dev --tmp -lruntime=trace -lruntime::contracts=debug -lerror
```
Use
- polkadot.JS. Instructions on [Astar docs](https://docs.astar.network/docs/build/wasm/tooling/polkadotjs)
- or [Contracts UI](https://contracts-ui.substrate.io/)

to deploy contract on the local Swanky node

##### 💫 Deployed contracts
Test on Shibuya - [XzoT9sH6zpC19TdkkePiopgXjTHcEgX8qjXXHs4p1HuQ5uR](https://shibuya.subscan.io/account/XzoT9sH6zpC19TdkkePiopgXjTHcEgX8qjXXHs4p1HuQ5uR)

#### 📚 Learn
Follow the [From Zero to ink! Hero](https://docs.astar.network/docs/build/wasm/from-zero-to-ink-hero/) tutorial tu learn how to build this smart contract




# RMRK Proxy

RMRK is designed in a way that all tokens in collection needs to be pre-minted and equipped with assets to be useful, which increases costs for a collection creator.
RMRK contract proxy enables lazy token minting on RMRK contract. The proxy contract has single method `mint` which does the following:
  - mints RMRK token
  - adds random asset to the token
  - transfers the token to the method caller

To be able to use this contract some prerequisites (see e2e test for details) must be met:
- RMRK and catalog contract deployed
- parts added to the catalog contract (`catalog::addPartList`)
- asset entries added to the RMRK contract (call `multiAsset::addAssetEntry` for each entry you want to add)


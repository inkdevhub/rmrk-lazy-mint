// Copyright (c) 2023 Astar Network
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the"Software"),
// to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#[ink::contract]
mod rmrk_proxy {
    use crate::{
        ensure,
        ProxyError,
        Result,
    };
    use ink::{
        env::{
            call::{
                build_call,
                ExecutionInput,
                Selector,
            },
            hash,
            DefaultEnvironment,
        },
        prelude::vec::Vec,
    };
    use openbrush::{
        contracts::{
            ownable::*,
            psp34::Id,
            reentrancy_guard::*,
        },
        modifiers,
        traits::Storage,
    };

    // Proxy contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct RmrkProxy {
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        guard: reentrancy_guard::Data,
        #[storage_field]
        proxy: crate::types::Data,
    }

    impl RmrkProxy {
        #[ink(constructor)]
        pub fn new(
            rmrk_contract: AccountId,
            catalog_contract: AccountId,
            mint_price: Balance,
        ) -> Self {
            let mut instance = Self::default();
            instance.proxy.rmrk_contract = Option::Some(rmrk_contract);
            instance.proxy.catalog_contract = Option::Some(catalog_contract);
            instance.proxy.salt = 0;
            instance.proxy.mint_price = mint_price;

            let caller = instance.env().caller();
            instance._init_with_owner(caller);
            instance
        }

        #[ink(message, payable)]
        #[modifiers(non_reentrant)]
        pub fn mint(&mut self) -> Result<()> {
            const GAS_LIMIT: u64 = 5_000_000_000;
            const MAX_ASSETS: u32 = 255;

            let transferred_value = Self::env().transferred_value();
            ensure!(
                transferred_value == self.proxy.mint_price,
                ProxyError::BadMintValue
            );

            let total_assets = build_call::<DefaultEnvironment>()
                .call(self.proxy.rmrk_contract.unwrap())
                .gas_limit(GAS_LIMIT)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "MultiAsset::total_assets"
                ))))
                .returns::<u32>()
                .try_invoke()
                .unwrap();
            ensure!(total_assets.unwrap() > 0, ProxyError::NoAssetsDefined);
            // This is temporary since current pseudo random generator is not working with big numbers.
            ensure!(
                total_assets.unwrap() <= MAX_ASSETS,
                ProxyError::TooManyAssetsDefined
            );

            // TODO check why the call is failing silently when no or invalid transferred value is provided.
            let mint_result = build_call::<DefaultEnvironment>()
                .call(self.proxy.rmrk_contract.unwrap())
                .gas_limit(GAS_LIMIT)
                .transferred_value(transferred_value)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "MintingLazy::mint"
                ))))
                .returns::<()>()
                .try_invoke();
            ink::env::debug_println!("mint_result: {:?}", mint_result);
            mint_result
                .map_err(|_| ProxyError::MintingError)?
                .map_err(|_| ProxyError::MintingError)?;

            let token_id = build_call::<DefaultEnvironment>()
                .call(self.proxy.rmrk_contract.unwrap())
                .gas_limit(GAS_LIMIT)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "PSP34::total_supply"
                ))))
                .returns::<u64>()
                .try_invoke()
                .unwrap()
                .unwrap();

            let asset_id = self.get_pseudo_random((total_assets.unwrap() - 1) as u8) + 1;
            let add_asset_result = build_call::<DefaultEnvironment>()
                .call(self.proxy.rmrk_contract.unwrap())
                .gas_limit(GAS_LIMIT)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        "MultiAsset::add_asset_to_token"
                    )))
                    .push_arg(Id::U64(token_id)) // TODO check if there is other way to determine token Id, beside reading totalSupply?
                    .push_arg(asset_id as u32)
                    .push_arg(None::<u32>),
                )
                .returns::<()>()
                .try_invoke()
                .map_err(|_| ProxyError::AddTokenAssetError)?;
            add_asset_result.map_err(|_| ProxyError::AddTokenAssetError)?;

            let caller = Self::env().caller();
            let transfer_token_result = build_call::<DefaultEnvironment>()
                .call(self.proxy.rmrk_contract.unwrap())
                .gas_limit(GAS_LIMIT)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP34::transfer")))
                        .push_arg(caller)
                        .push_arg(Id::U64(token_id))
                        .push_arg(Vec::<u8>::new()),
                )
                .returns::<()>()
                .try_invoke()
                .map_err(|_| ProxyError::OwnershipTransferError)?;
            transfer_token_result.map_err(|_| ProxyError::OwnershipTransferError)?;

            Ok(())
        }

        #[ink(message)]
        pub fn rmrk_contract_address(&self) -> AccountId {
            self.proxy.rmrk_contract.unwrap()
        }

        #[ink(message)]
        pub fn catalog_contract_address(&self) -> AccountId {
            self.proxy.catalog_contract.unwrap()
        }

        #[ink(message)]
        pub fn mint_price(&self) -> Balance {
            self.proxy.mint_price
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_rmrk_contract_address(&mut self, new_contract_address: AccountId) -> Result<()> {
            self.proxy.rmrk_contract = Option::Some(new_contract_address);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_catalog_contract_address(
            &mut self,
            new_contract_address: AccountId,
        ) -> Result<()> {
            self.proxy.catalog_contract = Option::Some(new_contract_address);
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_mint_price(&mut self, new_mint_price: Balance) -> Result<()> {
            self.proxy.mint_price = new_mint_price;
            Ok(())
        }

        /// Generates pseudo random number, Used to pick a random asset for a token.
        fn get_pseudo_random(&mut self, max_value: u8) -> u8 {
            let seed = self.env().block_timestamp();
            let mut input: Vec<u8> = Vec::new();
            input.extend_from_slice(&seed.to_be_bytes());
            input.extend_from_slice(&self.proxy.salt.to_be_bytes());
            let mut output = <hash::Keccak256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Keccak256>(&input, &mut output);
            self.proxy.salt += 1;
            output[0] % (max_value + 1)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        #[ink::test]
        fn constructor_works() {
            let contract = init_contract();
            assert_eq!(contract.rmrk_contract_address(), rmrk_address());
            assert_eq!(contract.catalog_contract_address(), catalog_address());
            assert_eq!(contract.mint_price(), 1_000_000_000_000_000_000);
        }

        #[ink::test]
        fn set_rmrk_contract_address_works() {
            let mut contract = init_contract();
            let new_rmrk: AccountId = [0x43; 32].into();
            assert!(contract.set_rmrk_contract_address(new_rmrk).is_ok());
            assert_eq!(contract.rmrk_contract_address(), new_rmrk);
        }

        #[ink::test]
        fn set_rmrk_contract_address_fails_if_not_owner() {
            let mut contract = init_contract();
            let new_rmrk: AccountId = [0x43; 32].into();
            set_sender(default_accounts().bob);
            assert_eq!(
                contract.set_rmrk_contract_address(new_rmrk),
                Err(ProxyError::OwnableError(OwnableError::CallerIsNotOwner))
            );
        }

        #[ink::test]
        fn set_catalog_contract_address_works() {
            let mut contract = init_contract();
            let new_rmrk: AccountId = [0x43; 32].into();
            assert!(contract.set_catalog_contract_address(new_rmrk).is_ok());
            assert_eq!(contract.catalog_contract_address(), new_rmrk);
        }

        #[ink::test]
        fn set_catalog_contract_address_fails_if_not_owner() {
            let mut contract = init_contract();
            let new_rmrk: AccountId = [0x43; 32].into();
            set_sender(default_accounts().bob);
            assert_eq!(
                contract.set_catalog_contract_address(new_rmrk),
                Err(ProxyError::OwnableError(OwnableError::CallerIsNotOwner))
            );
        }

        #[ink::test]
        fn set_mint_price_works() {
            let mut contract = init_contract();
            assert!(contract.set_mint_price(100).is_ok());
            assert_eq!(contract.mint_price(), 100);
        }

        #[ink::test]
        fn set_mint_price_fails_if_not_owner() {
            let mut contract = init_contract();
            set_sender(default_accounts().bob);
            assert_eq!(
                contract.set_mint_price(100),
                Err(ProxyError::OwnableError(OwnableError::CallerIsNotOwner))
            );
        }

        #[ink::test]
        fn mint_fails_if_no_balance() {
            let mut contract = init_contract();
            assert_eq!(contract.mint(), Err(ProxyError::BadMintValue));
        }

        fn init_contract() -> RmrkProxy {
            set_sender(default_accounts().alice);
            RmrkProxy::new(rmrk_address(), catalog_address(), 1_000_000_000_000_000_000)
        }

        fn rmrk_address() -> AccountId {
            AccountId::from([0x42; 32])
        }

        fn catalog_address() -> AccountId {
            AccountId::from([0x41; 32])
        }

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<Environment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use crate::proxy::rmrk_proxy::RmrkProxyRef;
        use catalog_example::catalog_example::CatalogContractRef;
        use ink::primitives::AccountId;
        use ink_e2e::build_message;
        use openbrush::contracts::psp34::{
            psp34_external::PSP34,
            Id,
        };
        use rmrk::{
            storage::catalog_external::Catalog,
            traits::multiasset_external::MultiAsset,
            types::{
                Part,
                PartType,
            },
        };
        use rmrk_equippable_lazy::rmrk_equippable_lazy::RmrkRef;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn mint_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let alice = ink_e2e::alice();

            // *************** Create catalog contract and add parts ***************
            let catalog_constructor = CatalogContractRef::new(String::from("ipfs://").into());
            let catalog_contract_address = client
                .instantiate("catalog_example", &alice, catalog_constructor, 0, None)
                .await
                .expect("Catalog contract instantiation failed")
                .account_id;

            // Add part to catalog
            let part_ids = vec![0];
            let parts = vec![Part {
                part_type: PartType::Fixed,
                z: 0,
                equippable: vec![],
                part_uri: String::from("ipfs://").into(),
                is_equippable_by_all: false,
            }];
            let add_part_message =
                build_message::<CatalogContractRef>(catalog_contract_address.clone())
                    .call(|catalog| catalog.add_part_list(part_ids.clone(), parts.clone()));
            client
                .call(&alice, add_part_message, 0, None)
                .await
                .expect("Add part failed");

            let read_parts_count_message =
                build_message::<CatalogContractRef>(catalog_contract_address.clone())
                    .call(|catalog| catalog.get_parts_count());
            let read_parts_count_result = client
                .call_dry_run(&alice, &read_parts_count_message, 0, None)
                .await
                .return_value();
            assert_eq!(read_parts_count_result, 1);

            // *************** Create RMRK contract and add asset entry ***************
            let rmrk_constructor = RmrkRef::new(
                String::from("Test").into(),
                String::from("TST").into(),
                String::from("ipfs://base").into(),
                None,
                1_000_000_000_000_000_000,
                String::from("ipfs://collection").into(),
                AccountId::try_from(alice.account_id().as_ref()).unwrap(),
                1,
            );
            let rmrk_address = client
                .instantiate("rmrk_equippable_lazy", &alice, rmrk_constructor, 0, None)
                .await
                .expect("RMRK contract instantiation failed")
                .account_id;

            // Add asset to RMRK contract.
            let add_asset_entry_message =
                build_message::<RmrkRef>(rmrk_address.clone()).call(|rmrk| {
                    rmrk.add_asset_entry(
                        Some(catalog_contract_address.clone()),
                        1,
                        1,
                        String::from("ipfs://parturi").into(),
                        vec![0],
                    )
                });
            client
                .call(&alice, add_asset_entry_message, 0, None)
                .await
                .expect("Add asset entry failed");

            let read_assets_count_message =
                build_message::<RmrkRef>(rmrk_address.clone()).call(|rmrk| rmrk.total_assets());
            let read_assets_count_result = client
                .call_dry_run(&alice, &read_assets_count_message, 0, None)
                .await
                .return_value();
            assert_eq!(read_assets_count_result, 1);

            // *************** Create RMRK proxy contract and mint ***************
            let proxy_constructor = RmrkProxyRef::new(
                rmrk_address,
                catalog_contract_address,
                1_000_000_000_000_000_000,
            );
            let proxy_address = client
                .instantiate("rmrk_proxy", &alice, proxy_constructor, 0, None)
                .await
                .expect("Proxy contract instantiation failed")
                .account_id;

            // Mint token.
            let mint_message =
                build_message::<RmrkProxyRef>(proxy_address.clone()).call(|proxy| proxy.mint());
            client
                .call(&alice, mint_message, 1_000_000_000_000_000_000, None)
                .await
                .expect("Mint failed");

            // Check if token was minted
            let read_total_supply_message =
                build_message::<RmrkRef>(rmrk_address.clone()).call(|rmrk| rmrk.total_supply());

            let read_total_supply_result = client
                .call_dry_run(&alice, &read_total_supply_message, 0, None)
                .await
                .return_value();
            assert_eq!(read_total_supply_result, 1);

            // Check if asset has been added to the token.
            let read_total_assets_message = build_message::<RmrkRef>(rmrk_address.clone())
                .call(|rmrk| rmrk.total_token_assets(Id::U64(1)));

            let read_total_assets_result = client
                .call_dry_run(&alice, &read_total_assets_message, 0, None)
                .await
                .return_value()
                .unwrap();
            // ink::env::debug_println!("token assets: {:?}", read_total_assets_result);
            assert_eq!(read_total_assets_result.0, 1);

            // Check if token owner is same as the caller.
            let read_owner_of_message = build_message::<RmrkRef>(rmrk_address.clone())
                .call(|rmrk| rmrk.owner_of(Id::U64(1)));

            let read_owner_of_result = client
                .call_dry_run(&alice, &read_owner_of_message, 0, None)
                .await
                .return_value()
                .unwrap();

            let alice_account_id_32 = alice.account_id();
            let alice_account_id = AccountId::try_from(alice_account_id_32.as_ref()).unwrap();
            assert_eq!(read_owner_of_result, alice_account_id);

            Ok(())
        }
    }
}

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into())
        }
    }};
}

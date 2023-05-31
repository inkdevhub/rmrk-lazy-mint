use openbrush::{
    contracts::{
        ownable::OwnableError,
        reentrancy_guard::ReentrancyGuardError,
    },
    traits::{
        AccountId,
        Balance,
    },
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub rmrk_contract: Option<AccountId>, //RMRK contract address
    pub catalog_contract: Option<AccountId>, // Catalog contract address
    pub mint_price: Balance, // A token minting price
    pub salt: u64, // used for pseudo random number generation
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ProxyError {
    /// Error happened while trying to add asset to minted token. 
    AddTokenAssetError,
    // A value passed to mint method doesn't match mint_price.
    BadMintValue,
    /// Something went wrong while invoking mint method on the RMRK contract.
    MintingError,
    /// No assets defined on RMRK contract.
    NoAssetsDefined,
    /// A caller is not a marketplace owner.
    OwnableError(OwnableError),
    /// Error happened while trying to transfer minted token ownership to a caller.
    OwnershipTransferError,
    /// A caller is trying to make second call while 1st one is still executing.
    ReentrancyError(ReentrancyGuardError),
    /// Too many assets defined on RMRK contract. This is a limitation of the current proxy implementation
    /// where get_pseudo_random function returns u8.
    TooManyAssetsDefined,
}

pub type Result<T> = core::result::Result<T, ProxyError>;

impl From<OwnableError> for ProxyError {
    fn from(error: OwnableError) -> Self {
        ProxyError::OwnableError(error)
    }
}

impl From<ReentrancyGuardError> for ProxyError {
    fn from(error: ReentrancyGuardError) -> Self {
        ProxyError::ReentrancyError(error)
    }
}

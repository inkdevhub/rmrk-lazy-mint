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
    pub rmrk_contract: Option<AccountId>,
    pub catalog_contract: Option<AccountId>,
    pub mint_price: Balance,
    pub salt: u64, // used for pseudo random number generation
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ProxyError {
    /// A caller is not a marketplace owner.
    OwnableError(OwnableError),
    /// A caller is trying to make second call while 1st one is still executing.
    ReentrancyError(ReentrancyGuardError),
    /// Something went wrong while invoking mint method on the proxied contract.
    MintingError,
    OwnershipTransferError,
    AddTokenAssetError,
    NoAssetsDefined,
    TooManyAssetsDefined,
    BadMintValue,
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

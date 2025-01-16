use num_enum::{FromPrimitive, IntoPrimitive};
use solana_sdk::program_error::ProgramError;
use thiserror::Error;

pub type DexResult<T = ()> = Result<T, DexError>;

#[derive(Debug)]
pub struct AssertionError {
    pub line: u16,
    pub file_id: SourceFileId,
}

impl From<AssertionError> for u32 {
    fn from(err: AssertionError) -> u32 {
        (err.line as u32) + ((err.file_id as u8 as u32) << 24)
    }
}

impl From<AssertionError> for DexError {
    fn from(err: AssertionError) -> DexError {
        let err: u32 = err.into();
        DexError::ProgramError(ProgramError::Custom(err.into()))
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DexError {
    #[error(transparent)]
    ProgramError(#[from] ProgramError),
    #[error("{0:?}")]
    ErrorCode(#[from] DexErrorCode),
}

#[derive(Debug, IntoPrimitive, FromPrimitive, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DexErrorCode {
    InvalidMarketFlags = 0,
    InvalidAskFlags,
    InvalidBidFlags,
    InvalidQueueLength,
    OwnerAccountNotProvided,

    ConsumeEventsQueueFailure,
    WrongCoinVault,
    WrongPcVault,
    WrongCoinMint,
    WrongPcMint,

    CoinVaultProgramId = 10,
    PcVaultProgramId,
    CoinMintProgramId,
    PcMintProgramId,

    WrongCoinMintSize,
    WrongPcMintSize,
    WrongCoinVaultSize,
    WrongPcVaultSize,

    UninitializedVault,
    UninitializedMint,

    CoinMintUninitialized = 20,
    PcMintUninitialized,
    WrongMint,
    WrongVaultOwner,
    VaultHasDelegate,

    AlreadyInitialized,
    WrongAccountDataAlignment,
    WrongAccountDataPaddingLength,
    WrongAccountHeadPadding,
    WrongAccountTailPadding,

    RequestQueueEmpty = 30,
    EventQueueTooSmall,
    SlabTooSmall,
    BadVaultSignerNonce,
    InsufficientFunds,

    SplAccountProgramId,
    SplAccountLen,
    WrongFeeDiscountAccountOwner,
    WrongFeeDiscountMint,

    CoinPayerProgramId,
    PcPayerProgramId = 40,
    ClientIdNotFound,
    TooManyOpenOrders,

    FakeErrorSoWeDontChangeNumbers,
    BorrowError,

    WrongOrdersAccount,
    WrongBidsAccount,
    WrongAsksAccount,
    WrongRequestQueueAccount,
    WrongEventQueueAccount,

    RequestQueueFull = 50,
    EventQueueFull,
    MarketIsDisabled,
    WrongSigner,
    TransferFailed,
    ClientOrderIdIsZero,

    WrongRentSysvarAccount,
    RentNotProvided,
    OrdersNotRentExempt,
    OrderNotFound,
    OrderNotYours = 60,

    WouldSelfTrade,
    InvalidOpenOrdersAuthority,
    OrderMaxTimestampExceeded,

    Unknown = 1000,

    // This contains the line number in the lower 16 bits,
    // and the source file id in the upper 8 bits
    #[num_enum(default)]
    AssertionError,
}

#[repr(u8)]
#[derive(Error, Debug)]
pub enum SourceFileId {
    #[error("src/state.rs")]
    State = 1,
    #[error("src/matching.rs")]
    Matching = 2,
    #[error("src/critbit.rs")]
    Critbit = 3,
}

#[macro_export]
macro_rules! declare_check_assert_macros {
    ($source_file_id:expr) => {
        macro_rules! assertion_error {
            () => {{
                let file_id: SourceFileId = $source_file_id;
                $crate::error::AssertionError {
                    line: line!() as u16,
                    file_id,
                }
            }};
        }

        #[allow(unused_macros)]
        macro_rules! check_assert {
            ($val:expr) => {{
                if $val {
                    Ok(())
                } else {
                    Err(assertion_error!())
                }
            }};
        }

        #[allow(unused_macros)]
        macro_rules! check_assert_eq {
            ($a:expr, $b:expr) => {{
                if $a == $b {
                    Ok(())
                } else {
                    Err(assertion_error!())
                }
            }};
        }

        #[allow(unused_macros)]
        macro_rules! check_unreachable {
            () => {{
                Err(assertion_error!())
            }};
        }
    };
}

impl std::fmt::Display for DexErrorCode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        <Self as std::fmt::Debug>::fmt(self, fmt)
    }
}

impl std::error::Error for DexErrorCode {}

impl std::convert::From<DexError> for ProgramError {
    fn from(e: DexError) -> ProgramError {
        match e {
            DexError::ProgramError(e) => e,
            DexError::ErrorCode(c) => ProgramError::Custom(c.into()),
        }
    }
}

impl std::convert::From<std::cell::BorrowError> for DexError {
    fn from(_: std::cell::BorrowError) -> Self {
        DexError::ErrorCode(DexErrorCode::BorrowError)
    }
}
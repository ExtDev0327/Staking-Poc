use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum NFTStakingError {
    /// The account cannot be initialized because it is already being used.
    #[error("AlreadyInUse")]
    AlreadyInUse,
    /// Required signature is missing.
    #[error("SignatureMissing")]
    SignatureMissing,
    /// Invalid Instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// Invalid stake list account.
    #[error("InvalidStakeList")]
    InvalidStakeList,
    /// Stake account for this nft not found in the list.
    #[error("StakedNFTNotFound")]
    StakedNFTNotFound,
    /// The deserialization of the account returned something besides State::Account.
    #[error("Deserialized account is not an SPL Token account")]
    ExpectedAccount,
}

impl From<NFTStakingError> for ProgramError {
    fn from(e: NFTStakingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

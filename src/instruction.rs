use crate::error::NFTStakingError::InvalidInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock, rent},
};
use std::convert::TryInto;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DepositNFTData {
    pub amount: u64,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum NFTStakingInstruction {
    Initialize,
    DepositNFT(DepositNFTData),
    WithdrawNFT,
}

impl NFTStakingInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::DepositNFT(DepositNFTData {
                amount: Self::unpack_u64(rest)?,
            }),
            2 => Self::WithdrawNFT,
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_u64(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }

    // fn unpack_u16(input: &[u8]) -> Result<u16, ProgramError> {
    //     let max_items = input
    //         .get(..8)
    //         .and_then(|slice| slice.try_into().ok())
    //         .map(u16::from_le_bytes)
    //         .ok_or(InvalidInstruction)?;
    //     Ok(max_items)
    // }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match *self {
            Self::Initialize => buf.push(0),
            Self::DepositNFT(DepositNFTData { amount }) => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::WithdrawNFT => buf.push(1),
        }
        buf
    }
}

/// creates a 'initialize' instruction
pub fn initialize(
    program_id: &Pubkey,
    stake_store_pubkey: &Pubkey,
    stake_list_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = NFTStakingInstruction::Initialize.pack();
    let accounts = vec![
        AccountMeta::new(*stake_store_pubkey, false),
        AccountMeta::new(*stake_list_pubkey, false),
        AccountMeta::new_readonly(*owner_pubkey, true),
        AccountMeta::new_readonly(rent::id(), false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// creates a 'deposit_nft' instruction
pub fn deposit_nft(
    program_id: &Pubkey,
    depositor_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    stake_pubkey: &Pubkey,
    stake_store_pubkey: &Pubkey,
    stake_list_pubkey: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let data = NFTStakingInstruction::DepositNFT(DepositNFTData { amount }).pack();
    let accounts = vec![
        AccountMeta::new(*depositor_pubkey, true),
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new(clock::id(), false),
        AccountMeta::new(*stake_pubkey, false),
        AccountMeta::new(*stake_store_pubkey, false),
        AccountMeta::new(*stake_list_pubkey, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

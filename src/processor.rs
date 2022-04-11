use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    clock::Clock,
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::{PrintProgramError, ProgramError},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    // system_instruction,
    // system_program,
    sysvar::Sysvar,
};

use crate::{
    error::NFTStakingError,
    instruction::{DepositNFTData, NFTStakingInstruction},
    state::{
        StakeList,
        StakeListHeader,
        StakeStore,
        StakedNFT,
    },
    utils::{ unpack_token_account, MAX_ITEMS },
};
use num_traits::FromPrimitive;

use spl_token::state::Account as TokenAccount;

const TRANSIENT_NFT_STAKE_SEED_PREFIX: &[u8] = b"transient";

// /// Check system program address
// fn check_system_program(program_id: &Pubkey) -> Result<(), ProgramError> {
//     if *program_id != system_program::id() {
//         msg!(
//             "Expected system program {}, received {}",
//             system_program::id(),
//             program_id
//         );
//         Err(ProgramError::IncorrectProgramId)
//     } else {
//         Ok(())
//     }
// }

// /// Check stake program address
// fn check_stake_program(program_id: &Pubkey) -> Result<(), ProgramError> {
//     if *program_id != crate::id() {
//         msg!(
//             "Expected nft staking poc program {}, received {}",
//             crate::id(),
//             program_id
//         );
//         Err(ProgramError::IncorrectProgramId)
//     } else {
//         Ok(())
//     }
// }

/// Check account owner is the given program
fn check_account_owner(
    account_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if *program_id != *account_info.owner {
        msg!(
            "Expected account to be owned by program {}, received {}",
            program_id,
            account_info.owner
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = NFTStakingInstruction::unpack(instruction_data)?;

        match instruction {
            NFTStakingInstruction::Initialize => {
                msg!("Instruction: Initialize");
                Self::process_initialize(accounts, program_id)
            }
            NFTStakingInstruction::DepositNFT(DepositNFTData { amount }) => {
                msg!("Instruction: DepositNFT");
                Self::process_deposit_nft(accounts, amount, program_id)
            }
            NFTStakingInstruction::WithdrawNFT => {
                msg!("Instruction: WithdrawNFT");
                Self::process_withdraw_nft(accounts, program_id)
            }
        }
    }

    fn process_initialize(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_store_info = next_account_info(account_info_iter)?;
        let stake_list_info = next_account_info(account_info_iter)?;
        let manager_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;

        let rent = &Rent::from_account_info(rent_info)?;
        if !manager_info.is_signer {
            msg!("Manager did not sign to initialize");
            return Err(NFTStakingError::SignatureMissing.into());
        }

        if stake_store_info.key == stake_list_info.key {
            msg!("Can't use the same account for stake store and stake list");
            return Err(NFTStakingError::AlreadyInUse.into());
        }

        check_account_owner(stake_store_info, program_id)?;
        let mut stake_store =
            try_from_slice_unchecked::<StakeStore>(&stake_store_info.data.borrow())?;

        check_account_owner(stake_list_info, program_id)?;
        let mut stake_list = try_from_slice_unchecked::<StakeList>(&stake_list_info.data.borrow())?;

        // should not exceed (10M - header'size) / StakedNFT's size. 100K items will be possible as ideal.
        stake_list.header.is_initialized = true;
        stake_list.header.max_items = MAX_ITEMS;
        stake_list.header.count = 0;
        stake_list.items.clear();

        if !rent.is_exempt(stake_store_info.lamports(), stake_store_info.data_len()) {
            msg!("Stake store not rent-exempt");
            return Err(ProgramError::AccountNotRentExempt);
        }

        if !rent.is_exempt(stake_list_info.lamports(), stake_list_info.data_len()) {
            msg!("Stake list not rent-exempt");
            return Err(ProgramError::AccountNotRentExempt);
        }
        stake_list.serialize(&mut *stake_list_info.data.borrow_mut())?;

        let stake_store = StakeStore {
            is_initialized: true,
            manager: *manager_info.key,
            stake_list: *stake_list_info.key,
            staked_count: 0,
        };

        stake_store
            .serialize(&mut *stake_store_info.data.borrow_mut())
        .map_err(|e| e.into())
    }

    fn process_deposit_nft(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let depositor_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;

        if !depositor_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let stake_account_info = next_account_info(account_info_iter)?;
        // check to balance of stake_account
        // ...

        let stake_store_info = next_account_info(account_info_iter)?;
        check_account_owner(stake_store_info, program_id)?;
        let mut stake_store =
            try_from_slice_unchecked::<StakeStore>(&stake_store_info.data.borrow())?;
        msg!("=========================");
        let stake_list_info = next_account_info(account_info_iter)?;
        check_account_owner(stake_list_info, program_id)?;
        stake_store.check_stake_list(stake_list_info)?;
        msg!("process_deposit_nft {}", depositor_info.key);

        let mut stake_list_data = stake_list_info.data.borrow_mut();
        let (mut header, mut stake_list) = StakeListHeader::deserialize_vec(&mut stake_list_data)?;
        if header.max_items == stake_list.len() as u16 {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let (pda, _nonce) = Pubkey::find_program_address(
            &[
                TRANSIENT_NFT_STAKE_SEED_PREFIX,
                &depositor_info.key.to_bytes(),
                &mint_info.key.to_bytes(),
            ],
            program_id,
        );

        let token_program = next_account_info(account_info_iter)?;
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            stake_account_info.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            depositor_info.key,
            &[&depositor_info.key],
        )?;

        invoke(
            &owner_change_ix,
            &[
                stake_account_info.clone(),
                depositor_info.clone(),
                token_program.clone(),
            ],
        )?;

        stake_list.push(StakedNFT {
            owner: *depositor_info.key,
            token_mint: *mint_info.key,
            holder: *stake_account_info.key,
            stake_time: clock.unix_timestamp,
        })?;

        // increase the stake_store's staked_count
        stake_store.staked_count += amount as u16;
        header.count += amount as u16;

        stake_store
            .serialize(&mut *stake_store_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    fn process_withdraw_nft(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let withdrawer_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let _clock = &Clock::from_account_info(clock_info)?;

        if !withdrawer_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let stake_store_info = next_account_info(account_info_iter)?;
        check_account_owner(stake_store_info, program_id)?;
        let stake_store = try_from_slice_unchecked::<StakeStore>(&stake_store_info.data.borrow())?;
        let stake_list_info = next_account_info(account_info_iter)?;
        check_account_owner(stake_list_info, program_id)?;
        stake_store.check_stake_list(&stake_list_info)?;

        let mut stake_list_data = stake_list_info.data.borrow_mut();
        let (_header, mut stake_list) = StakeListHeader::deserialize_vec(&mut stake_list_data)?;
        if 0 == stake_list.len() as u16 {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let stake_account_info = next_account_info(account_info_iter)?;

        let staked_nft_info: StakedNFT;
        {
            let maybe_staked_nft = stake_list.find_double::<StakedNFT>(
                withdrawer_info.key.as_ref(),
                mint_info.key.as_ref(),
                StakedNFT::memcmp_pubkey,
            );
            if maybe_staked_nft.is_none() {
                msg!(
                    "owner account {}, token mint {} not found in stake list",
                    withdrawer_info.key,
                    mint_info.key
                );
                return Err(NFTStakingError::StakedNFTNotFound.into());
            }

            staked_nft_info = *maybe_staked_nft.unwrap();
        }
        if staked_nft_info.holder != *stake_account_info.key {
            msg!(
                "owner {} or token mint {} mismatch for staked NFT",
                withdrawer_info.key,
                mint_info.key
            );
            return Err(NFTStakingError::StakedNFTNotFound.into());
        }

        let stake_account = TokenAccount::unpack(&stake_account_info.data.borrow())?;
        let (pda, nonce) = Pubkey::find_program_address(
            &[
                TRANSIENT_NFT_STAKE_SEED_PREFIX,
                &withdrawer_info.key.to_bytes(),
                &mint_info.key.to_bytes(),
            ],
            program_id,
        );
        let pda_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let transfer_to_withrawer_ix = spl_token::instruction::transfer(
            token_program.key,
            stake_account_info.key,
            withdrawer_info.key,
            &pda,
            &[&pda],
            stake_account.amount,
        )?;

        let authority_signature_seeds: &[&[u8]] = &[
            &TRANSIENT_NFT_STAKE_SEED_PREFIX[..],
            &withdrawer_info.key.to_bytes()[..],
            &mint_info.key.to_bytes()[..],
            &[nonce],
        ];

        msg!("Calling the token program to transfer token to the withdrawer...");
        invoke_signed(
            &transfer_to_withrawer_ix,
            &[
                stake_account_info.clone(),
                withdrawer_info.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[authority_signature_seeds],
        )?;

        stake_list
            .retain::<StakedNFT>(StakedNFT::is_not_withdrawn, staked_nft_info.holder.as_ref())?;

        // decrease the stake_store's staked_count
        // ...

        let close_pdas_stake_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            stake_account_info.key,
            withdrawer_info.key,
            &pda,
            &[&pda],
        )?;
        msg!("Calling the token program to close stake account which has been withdrawn...");
        invoke_signed(
            &close_pdas_stake_acc_ix,
            &[
                stake_account_info.clone(),
                withdrawer_info.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[authority_signature_seeds],
        )?;

        msg!("Closing the stake account...");
        Ok(())
    }
}

impl PrintProgramError for NFTStakingError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            NFTStakingError::AlreadyInUse => {
                msg!("Error: The account cannot be initialized because it is already being used")
            }
            NFTStakingError::AmountOverflow => {
                msg!("Error: Staking has been reached maximum count")
            }
            NFTStakingError::ExpectedAmountMismatch => msg!("Error: Expected Amount Mismatch"),
            NFTStakingError::InvalidInstruction => msg!("Error: Invalid Instruction"),
            NFTStakingError::InvalidStakeList => msg!("Error: Detect mismatching of Stake List"),
            NFTStakingError::NotRentExempt => msg!("Error: Not Rent Exempt"),
            NFTStakingError::SignatureMissing => msg!("Error: Required signature is missing"),
            NFTStakingError::StakedNFTNotFound => {
                msg!("Error: Stake account for this nft not found in the list")
            }
            NFTStakingError::ExpectedAccount => {
                msg!("The deserialization of the account returned something besides State::Account")
            }
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::{
        instruction::{deposit_nft, initialize},
        state::{StakeList, StakeListHeader, StakeStore, StakedNFT},
        utils::{test_utils::*, unpack_token_account, MAX_ITEMS},
    };
    use borsh::BorshSerialize;
    use solana_program::{
        borsh::{get_instance_packed_len, get_packed_len, try_from_slice_unchecked},
        clock::Epoch,
        msg,
        program_pack::Pack,
        rent::Rent,
        sysvar,
    };
    use solana_sdk::account::{create_account_for_test, Account, WritableAccount};
    use spl_token::{
        instruction::{initialize_account, initialize_mint, mint_to, transfer},
        state::{Account as SplAccount, Mint as SplMint},
    };

    #[test]
    fn test_deposit_nft() {
        msg!("starting test_deposit_nft {}", STAKE_PROGRAM_ID);
        let owner_key = pubkey_rand();
        let depositor_key = pubkey_rand();
        let stake_store_key = pubkey_rand();
        let stake_list_key = pubkey_rand();

        // setup accounts
        let mut depositor_account = Account::default();
        let mut stake_store_account = Account::new(
            Rent::default().minimum_balance(get_packed_len::<StakeStore>()),
            get_packed_len::<StakeStore>(),
            &STAKE_PROGRAM_ID,
        );

        let mut stake_store =
            try_from_slice_unchecked::<StakeStore>(&stake_store_account.data).unwrap();
        let mut test_data = vec![1; stake_store_account.data.len()];
        let mut fee = Rent::default().minimum_balance(get_packed_len::<StakeStore>());

        let stake_store_account_info = AccountInfo::new(
            &stake_store_key,
            false,
            true,
            &mut fee,
            test_data.as_mut_slice(),
            &depositor_key,
            false,
            Epoch::default(),
        );

        let list_size = get_packed_len::<StakedNFT>() * MAX_ITEMS as usize
            + get_packed_len::<StakeListHeader>(); //get_packed_len::<StakeList>();
                                                   // msg!("list_size: {}", list_size);
        let mut stake_list_account = Account::new(
            Rent::default().minimum_balance(list_size),
            list_size,
            &STAKE_PROGRAM_ID,
        );
        let (nft1_mint_key, mut nft1_mint_account) =
            create_mint(&spl_token::id(), &owner_key, DEFAULT_TOKEN_DECIMALS, None);
        let (nft2_mint_key, mut nft2_mint_account) =
            create_mint(&spl_token::id(), &owner_key, DEFAULT_TOKEN_DECIMALS, None);

        let (user_nft1_key, mut user_nft1_account) = mint_token(
            &spl_token::id(),
            &nft1_mint_key,
            &mut nft1_mint_account,
            &owner_key,
            &depositor_key,
            1,
        );

        let (user_nft2_key, mut user_nft2_account) = mint_token(
            &spl_token::id(),
            &nft2_mint_key,
            &mut nft2_mint_account,
            &owner_key,
            &depositor_key,
            1,
        );

        let (stake_nft1_key, mut stake_nft1_account) = mint_token(
            &spl_token::id(),
            &nft1_mint_key,
            &mut nft1_mint_account,
            &owner_key,
            &depositor_key,
            0,
        );

        let (stake_nft2_key, mut stake_nft2_account) = mint_token(
            &spl_token::id(),
            &nft2_mint_key,
            &mut nft2_mint_account,
            &owner_key,
            &depositor_key,
            0,
        );

        {
            let user_nft1 = unpack_token_account(&user_nft1_account.data).unwrap();
            assert_eq!(user_nft1.amount, 1);
            let user_nft2 = unpack_token_account(&user_nft2_account.data).unwrap();
            assert_eq!(user_nft1.amount, 1);
            let stake_nft1 = unpack_token_account(&stake_nft1_account.data).unwrap();
            assert_eq!(stake_nft1.amount, 0);
            let stake_nft2 = unpack_token_account(&stake_nft2_account.data).unwrap();
            assert_eq!(stake_nft2.amount, 0);
        }

        do_process_instruction(
            transfer(
                &spl_token::id(),
                &user_nft1_key,
                &stake_nft1_key,
                &depositor_key,
                &[],
                1,
            )
            .unwrap(),
            vec![
                &mut user_nft1_account,
                &mut stake_nft1_account,
                &mut depositor_account,
            ],
        )
        .unwrap();

        do_process_instruction(
            transfer(
                &spl_token::id(),
                &user_nft2_key,
                &stake_nft2_key,
                &depositor_key,
                &[],
                1,
            )
            .unwrap(),
            vec![
                &mut user_nft2_account,
                &mut stake_nft2_account,
                &mut depositor_account,
            ],
        )
        .unwrap();

        {
            let user_nft1 = unpack_token_account(&user_nft1_account.data).unwrap();
            assert_eq!(user_nft1.amount, 0);
            let user_nft2 = unpack_token_account(&user_nft2_account.data).unwrap();
            assert_eq!(user_nft1.amount, 0);
            let stake_nft1 = unpack_token_account(&stake_nft1_account.data).unwrap();
            assert_eq!(stake_nft1.amount, 1);
            let stake_nft2 = unpack_token_account(&stake_nft2_account.data).unwrap();
            assert_eq!(stake_nft2.amount, 1);
        }

        msg!("*** before init: {}", stake_store_account.data.len());
        msg!("{} {}", owner_key, stake_list_key);
        do_process_instruction(
            initialize(
                &STAKE_PROGRAM_ID,
                &stake_store_key,
                &stake_list_key,
                &owner_key,
            )
            .unwrap(),
            vec![
                &mut stake_store_account,
                &mut stake_list_account,
                &mut Account::default(),
                &mut create_account_for_test(&Rent::default()),
            ],
        )
        .unwrap();

        msg!("*** after init: {}", stake_store_account.data.len());
        msg!("=================================");

        do_process_instruction(
            deposit_nft(
                &STAKE_PROGRAM_ID,
                &depositor_key,
                &nft1_mint_key,
                &stake_nft1_key,
                &stake_store_key,
                &stake_list_key,
                1,
            )
            .unwrap(),
            vec![
                &mut depositor_account,
                &mut nft1_mint_account,
                &mut clock_account(ZERO_TS),
                &mut stake_nft1_account,
                &mut stake_store_account,
                &mut stake_list_account,
            ],
        )
        .unwrap();

        do_process_instruction(
            deposit_nft(
                &STAKE_PROGRAM_ID,
                &depositor_key,
                &nft2_mint_key,
                &stake_nft2_key,
                &stake_store_key,
                &stake_list_key,
                1,
            )
            .unwrap(),
            vec![
                &mut depositor_account,
                &mut nft2_mint_account,
                &mut clock_account(ZERO_TS),
                &mut stake_nft2_account,
                &mut stake_store_account,
                &mut stake_list_account,
            ],
        )
        .unwrap();

        // check stake store and stake list
        // ...
    }
}

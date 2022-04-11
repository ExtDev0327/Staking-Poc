use crate::error::NFTStakingError;
use solana_program::program_pack::Pack;
use spl_token::state::Account;

pub const MAX_ITEMS: u16 = 65535;

/// Unpacks a spl_token `Account`.
pub fn unpack_token_account(data: &[u8]) -> Result<Account, NFTStakingError> {
    Account::unpack(data).map_err(|_| NFTStakingError::ExpectedAccount)
}

#[cfg(test)]
pub mod test_utils {
    use crate::processor::Processor;
    use solana_program::{
        account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
        instruction::Instruction, msg, program_error::ProgramError, program_pack::Pack,
        program_stubs, pubkey::Pubkey, rent::Rent, sysvar,
    };
    use solana_sdk::account::{create_account_for_test, create_is_signer_account_infos, Account};
    use spl_token::{
        instruction::{approve, initialize_account, initialize_mint, mint_to},
        state::{Account as SplAccount, Mint as SplMint},
    };
    pub const STAKE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([3u8; 32]);
    // pub const TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array([1u8; 32]); //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
    pub const DEFAULT_TOKEN_DECIMALS: u8 = 0; // for nft
    pub const ZERO_TS: i64 = 0;

    pub fn clock_account(ts: i64) -> Account {
        let mut clock = Clock::default();
        clock.unix_timestamp = ts;
        Account::new_data(1, &clock, &sysvar::id()).unwrap()
    }

    pub fn pubkey_rand() -> Pubkey {
        Pubkey::new_unique()
    }

    struct TestSyscallStubs {}
    impl program_stubs::SyscallStubs for TestSyscallStubs {
        fn sol_invoke_signed(
            &self,
            instruction: &Instruction,
            account_infos: &[AccountInfo],
            signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            msg!("TestSyscallStubs::sol_invoke_signed()");

            let mut new_account_infos = vec![];

            // mimic check for token program in accounts
            if !account_infos.iter().any(|x| *x.key == spl_token::id()) {
                return Err(ProgramError::InvalidAccountData);
            }

            for meta in instruction.accounts.iter() {
                for account_info in account_infos.iter() {
                    if meta.pubkey == *account_info.key {
                        let mut new_account_info = account_info.clone();
                        for seeds in signers_seeds.iter() {
                            let signer =
                                Pubkey::create_program_address(&seeds, &STAKE_PROGRAM_ID).unwrap();
                            if *account_info.key == signer {
                                new_account_info.is_signer = true;
                            }
                        }
                        new_account_infos.push(new_account_info);
                    }
                }
            }

            spl_token::processor::Processor::process(
                &instruction.program_id,
                &new_account_infos,
                &instruction.data,
            )
        }
    }

    fn test_syscall_stubs() {
        use std::sync::Once;
        static ONCE: Once = Once::new();

        ONCE.call_once(|| {
            program_stubs::set_syscall_stubs(Box::new(TestSyscallStubs {}));
        });
    }

    pub fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut Account>,
    ) -> ProgramResult {
        test_syscall_stubs();

        // approximate the logic in the actual runtime which runs the instruction
        // and only updates accounts if the instruction is successful
        let mut account_clones = accounts.iter().map(|x| (*x).clone()).collect::<Vec<_>>();
        let mut meta = instruction
            .accounts
            .iter()
            .zip(account_clones.iter_mut())
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();
        let mut account_infos = create_is_signer_account_infos(&mut meta);
        let res = if instruction.program_id == STAKE_PROGRAM_ID {
            Processor::process(&instruction.program_id, &account_infos, &instruction.data)
        } else {
            spl_token::processor::Processor::process(
                &instruction.program_id,
                &account_infos,
                &instruction.data,
            )
        };

        if res.is_ok() {
            let mut account_metas = instruction
                .accounts
                .iter()
                .zip(accounts)
                .map(|(account_meta, account)| (&account_meta.pubkey, account))
                .collect::<Vec<_>>();
            for account_info in account_infos.iter_mut() {
                for account_meta in account_metas.iter_mut() {
                    if account_info.key == account_meta.0 {
                        let account = &mut account_meta.1;
                        account.owner = *account_info.owner;
                        account.lamports = **account_info.lamports.borrow();
                        account.data = account_info.data.borrow().to_vec();
                    }
                }
            }
        }
        res
    }

    pub fn mint_minimum_balance() -> u64 {
        Rent::default().minimum_balance(SplMint::get_packed_len())
    }

    pub fn account_minimum_balance() -> u64 {
        Rent::default().minimum_balance(SplAccount::get_packed_len())
    }

    pub fn mint_token(
        program_id: &Pubkey,
        mint_key: &Pubkey,
        mut mint_account: &mut Account,
        mint_authority_key: &Pubkey,
        account_owner_key: &Pubkey,
        amount: u64,
    ) -> (Pubkey, Account) {
        let account_key = pubkey_rand();
        let mut account_account = Account::new(
            account_minimum_balance(),
            SplAccount::get_packed_len(),
            &program_id,
        );
        let mut mint_authority_account = Account::default();
        let mut rent_sysvar_account = create_account_for_test(&Rent::free());

        do_process_instruction(
            initialize_account(&program_id, &account_key, &mint_key, account_owner_key).unwrap(),
            vec![
                &mut account_account,
                &mut mint_account,
                &mut mint_authority_account,
                &mut rent_sysvar_account,
            ],
        )
        .unwrap();

        if amount > 0 {
            do_process_instruction(
                mint_to(
                    &program_id,
                    &mint_key,
                    &account_key,
                    &mint_authority_key,
                    &[],
                    amount,
                )
                .unwrap(),
                vec![
                    &mut mint_account,
                    &mut account_account,
                    &mut mint_authority_account,
                ],
            )
            .unwrap();
        }

        (account_key, account_account)
    }

    pub fn create_mint(
        program_id: &Pubkey,
        authority_key: &Pubkey,
        decimals: u8,
        freeze_authority: Option<&Pubkey>,
    ) -> (Pubkey, Account) {
        let mint_key = pubkey_rand();
        let mut mint_account = Account::new(
            mint_minimum_balance(),
            SplMint::get_packed_len(),
            &program_id,
        );
        let mut rent_sysvar_account = create_account_for_test(&Rent::free());

        do_process_instruction(
            initialize_mint(
                &program_id,
                &mint_key,
                authority_key,
                freeze_authority,
                decimals,
            )
            .unwrap(),
            vec![&mut mint_account, &mut rent_sysvar_account],
        )
        .unwrap();

        (mint_key, mint_account)
    }
}

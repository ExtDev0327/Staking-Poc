use crate::{big_vec::BigVec, error::NFTStakingError};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    borsh::get_instance_packed_len,
    clock::UnixTimestamp,
    msg,
    program_error::ProgramError,
    program_memory::sol_memcmp,
    program_pack::{Pack, Sealed, IsInitialized},
    pubkey::Pubkey,
};

/// Number of bytes in a pubkey
pub const PUBKEY_BYTES: usize = 32;

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct StakeStore {
    pub is_initialized: bool,
    pub manager: Pubkey,
    pub staked_count: u16,
    pub stake_list: Pubkey,
}

impl StakeStore {
    pub fn check_stake_list(&self, stake_list_info: &AccountInfo) -> Result<(), ProgramError> {
        if *stake_list_info.key != self.stake_list {
            msg!(
                "Invalid stake list provided, expected {}, received {}",
                self.stake_list,
                stake_list_info.key
            );
            Err(NFTStakingError::InvalidStakeList.into())
        } else {
            Ok(())
        }
    }
}

impl IsInitialized for StakeStore {
    fn is_initialized(&self) -> bool {
        self.is_initialized == true
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct StakeList {
    pub header: StakeListHeader,
    pub items: Vec<StakedNFT>,
}

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct StakeListHeader {
    pub is_initialized: bool,
    pub max_items: u16,
    pub count: u16,
}

impl StakeList {
    pub fn new(max_items: u16) -> Self {
        Self {
            header: StakeListHeader {
                is_initialized: false,
                max_items,
                count: 0,
            },
            items: vec![StakedNFT::default(); max_items as usize],
        }
    }
}

impl StakeListHeader {
    // const LEN: usize = 3;

    /// Extracts the stake list into its header and internal BigVec
    pub fn deserialize_vec(data: &mut [u8]) -> Result<(Self, BigVec), ProgramError> {
        let mut data_mut = &data[..];
        let header = StakeListHeader::deserialize(&mut data_mut)?;
        let length = get_instance_packed_len(&header)?;

        let big_vec = BigVec {
            data: &mut data[length..],
        };
        Ok((header, big_vec))
    }
}

impl IsInitialized for StakeListHeader {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct StakedNFT {
    pub owner: Pubkey,
    pub token_mint: Pubkey,
    pub holder: Pubkey,
    pub stake_time: UnixTimestamp,
}

impl StakedNFT {
    /// Performs a very cheap comparison, for checking if this stake
    /// info matches the owner and token_mint
    pub fn memcmp_pubkey(
        data: &[u8],
        owner_address_bytes: &[u8],
        mint_address_bytes: &[u8],
    ) -> bool {
        sol_memcmp(
            &data[0..0 + PUBKEY_BYTES],
            owner_address_bytes,
            PUBKEY_BYTES,
        ) == 0
            && sol_memcmp(
                &data[32..32 + PUBKEY_BYTES],
                mint_address_bytes,
                PUBKEY_BYTES,
            ) == 0
    }

    pub fn is_not_withdrawn(data: &[u8], holder_address_bytes: &[u8]) -> bool {
        sol_memcmp(
            &data[64..64 + PUBKEY_BYTES],
            holder_address_bytes,
            PUBKEY_BYTES,
        ) != 0
    }
}

impl Sealed for StakedNFT {}

impl Pack for StakedNFT {
    const LEN: usize = 32 * 3 + 8;
    fn pack_into_slice(&self, data: &mut [u8]) {
        let mut data = data;
        self.serialize(&mut data).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let unpacked = Self::try_from_slice(src)?;
        Ok(unpacked)
    }
}

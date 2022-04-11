pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod big_vec;
pub mod utils;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub use solana_program;

solana_program::declare_id!("NFTStakin1111111111111111111111111111111111");
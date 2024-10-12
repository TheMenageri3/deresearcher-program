pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("BdtzNv4J5DSCA52xK6KLyKG5qorajuwfmJV2WivPkRsW");

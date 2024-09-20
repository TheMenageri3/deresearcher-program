pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("P1SsZEQvb6gTPrdJQ5mu6oCyJCJhVKxFFnk9ztjsoEL");

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

solana_program::declare_id!("C5M2JxBaxmsW62BgujPXEPytw65igtUjr6mFbD5pmypM");

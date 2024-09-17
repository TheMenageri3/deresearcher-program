use std::error;

use solana_program::{
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DeResearcherError {
    #[error("Invalid Instruction (this ix is not supported)")]
    InvalidInstruction,
    #[error("Invalid Signer")]
    InvalidSigner,
    #[error("Paper already exists")]
    PaperAlreadyExists,
    #[error("Pubkey mismatch")]
    PubkeyMismatch,
    #[error("Invalid state")]
    InvalidState,
    #[error("Not enough approvals")]
    NotEnoughApprovals,
    #[error("Peer Review already exists")]
    PeerReviewAlreadyExists,
    #[error("Invalid Fee Receiver")]
    InvalidFeeReceiver,
}

impl From<DeResearcherError> for ProgramError {
    fn from(e: DeResearcherError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl PrintProgramError for DeResearcherError {
    fn print<E>(&self) {
        msg!("Error: {:?}", self);
    }
}

use solana_program::{
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DeResearcherError {
    #[error("Invalid Instruction (this ix is not supported)")]
    InvalidInstruction,
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

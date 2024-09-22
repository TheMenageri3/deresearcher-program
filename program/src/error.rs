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
    #[error("Profile already exists")]
    ResearcherProfileAlreadyExists,
    #[error("Profile not found")]
    ResearcherProfileNotFound,
    #[error("Not allowed for peer review")]
    NotAllowedForPeerReview,
    #[error("Paper not found")]
    PaperNotFound,
    #[error("serialization error")]
    SerializationError,
    #[error("Size overflow")]
    SizeOverflow,
    #[error("Account is Immutable")]
    ImmutableAccount,
    #[error("PDA pubkey mismatch")]
    PdaPubekyMismatch,
    #[error("Publisher cannot add a peer review to their own paper")]
    PublisherCannotAddPeerReview,
    #[error("Invalid Reputation checker")]
    InvalidReputationChecker,
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

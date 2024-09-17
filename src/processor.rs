use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    error::DeResearcherError,
    ix::{
        add_peer_review_ix, create_researche_paper_ix, get_access_ix, publish_paper_ix,
        DeResearcherInstruction,
    },
};

pub struct Processor {}

impl Processor {
    pub fn process_ix(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = DeResearcherInstruction::try_from_slice(instruction_data)
            .map_err(|_| DeResearcherError::InvalidInstruction)?;

        match instruction {
            DeResearcherInstruction::CreateResearchePaper(data) => {
                create_researche_paper_ix(program_id, accounts, data)?;
            }
            DeResearcherInstruction::PublishPaper => publish_paper_ix(program_id, accounts)?,
            DeResearcherInstruction::AddPeerReview(data) => {
                add_peer_review_ix(program_id, accounts, data)?;
            }
            DeResearcherInstruction::GetAccessToPaper(data) => {
                get_access_ix(program_id, accounts, data)?
            }
            _ => return Err(DeResearcherError::InvalidInstruction.into()),
        }

        Ok(())
    }
}

use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    error::DeResearcherError,
    instruction::{
        add_peer_review_ix, create_research_paper_ix, create_researcher_profile_ix, get_access_ix,
        publish_paper_ix, DeResearcherInstruction,
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
            DeResearcherInstruction::CreateResearcherProfile(data) => {
                create_researcher_profile_ix(program_id, accounts, data)?;
            }
            DeResearcherInstruction::CreateResearchePaper(data) => {
                create_research_paper_ix(program_id, accounts, data)?;
            }
            DeResearcherInstruction::PublishPaper(data) => {
                publish_paper_ix(program_id, accounts, data)?
            }
            DeResearcherInstruction::AddPeerReview(data) => {
                add_peer_review_ix(program_id, accounts, data)?;
            }
            DeResearcherInstruction::GetAccessToPaper(data) => {
                get_access_ix(program_id, accounts, data)?
            }
        }

        Ok(())
    }
}

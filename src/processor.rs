use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{error::DeResearcherError, ix::DeResearcherInstruction};

struct Processor {}

impl Processor {
    pub fn process_ix(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = DeResearcherInstruction::try_from_slice(instruction_data)
            .map_err(|_| DeResearcherError::InvalidInstruction);

        Ok(())
    }
}

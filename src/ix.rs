use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct CreateResearchePaper {}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum DeResearcherInstruction {
    CreateResearchePaper(CreateResearchePaper),
}

use std::vec;

use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    error::DeResearcherError,
    state::{
        PeerReview,
        ResearchMintCollection,
        ResearchPaper,
        ResearcherProfile,
        // ResearcherProfileState,
    },
};

const RESEARCH_PAPER_PDA_SEED: &[u8] = b"deres_research_paper";
const PEER_REVIEW_PDA_SEED: &[u8] = b"deres_peer_review";

const RESEARCH_MINT_COLLECTION_PDA_SEED: &[u8] = b"deres_mint_collection";

const RESEARCHER_PROFILE_PDA_SEED: &[u8] = b"deres_researcher_profile";

pub const MAX_REPUTATION: u8 = 100;

pub const MIN_REPUTATION_FOR_PEER_REVIEW: u8 = 50;

pub const MAX_STRING_SIZE: usize = 64;

// TODO: change this to 5
pub const MIN_APPROVALS_FOR_PUBLISH: u8 = 1;

pub const REPUTATION_CHECKER_ADDR: [u8; 32] = [
    169, 0, 98, 218, 109, 191, 169, 52, 91, 62, 13, 120, 87, 111, 105, 218, 157, 129, 43, 117, 250,
    6, 176, 236, 145, 237, 44, 88, 60, 29, 189, 169,
];

pub const REPUTATION_CHECKER_PUBKEY: Pubkey = Pubkey::new_from_array(REPUTATION_CHECKER_ADDR);

pub fn validate_pda(
    seeds: Vec<&[u8]>,
    pda: &Pubkey,
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), DeResearcherError> {
    let mut seeds_with_bump: Vec<&[u8]> = Vec::new();

    for seed in seeds {
        seeds_with_bump.push(seed);
    }

    let binding = [bump];

    seeds_with_bump.push(&binding);

    let actual_pda = Pubkey::create_program_address(&seeds_with_bump, program_id)
        .map_err(|_| DeResearcherError::PdaPubekyMismatch)?;

    if actual_pda.ne(pda) {
        return Err(DeResearcherError::PdaPubekyMismatch.into());
    }

    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreateResearcherProfile {
    pub name: String,
    pub meta_data_merkle_root: String,
    pub pda_bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreateResearchePaper {
    pub access_fee: u32,
    pub paper_content_hash: String,
    pub meta_data_merkle_root: String,
    pub pda_bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PublishPaper {
    pda_bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddPeerReview {
    pub quality_of_research: u8,
    pub potential_for_real_world_use_case: u8,
    pub domain_knowledge: u8,
    pub practicality_of_result_obtained: u8,
    pub meta_data_merkle_root: String,
    pub pda_bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MintResearchPaper {
    pub meta_data_merkle_root: String,
    pub pda_bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CheckAndAssignReputation {
    pub reputation: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, ShankInstruction)]
pub enum DeResearcherInstruction {
    #[account(
        0,
        writable,
        signer,
        name = "researcher_acc",
        desc = "Researcher's account"
    )]
    #[account(
        1,
        writable,
        name = "researcher_profile_pda_acc",
        desc = "Researcher's profile PDA account"
    )]
    #[account(2, name = "system_program_acc", desc = "System program account")]
    CreateResearcherProfile(CreateResearcherProfile),
    #[account(
        0,
        writable,
        signer,
        name = "publisher_acc",
        desc = "Publisher's account"
    )]
    #[account(
        1,
        writable,
        name = "researcher_profile_pda_acc",
        desc = "Researcher's profile PDA account"
    )]
    #[account(2, writable, name = "paper_pda_acc", desc = "Research paper account")]
    #[account(3, name = "system_program_acc", desc = "System program account")]
    CreateResearchePaper(CreateResearchePaper),
    #[account(
        0,
        writable,
        signer,
        name = "publisher_acc",
        desc = "Publisher's account"
    )]
    #[account(
        1,
        writable,
        name = "paper_pda_acc",
        desc = "Research paper PDA account"
    )]
    PublishPaper(PublishPaper),
    #[account(
        0,
        writable,
        signer,
        name = "reviewer_acc",
        desc = "Reviewer's account"
    )]
    #[account(
        1,
        writable,
        name = "researcher_profile_pda_acc",
        desc = "Researcher's profile PDA account"
    )]
    #[account(
        2,
        writable,
        name = "paper_pda_acc",
        desc = "Research paper PDA account"
    )]
    #[account(
        3,
        writable,
        name = "peer_review_pda_acc",
        desc = "Peer review PDA account"
    )]
    #[account(4, name = "system_program_acc", desc = "System program account")]
    AddPeerReview(AddPeerReview),
    #[account(0, writable, signer, name = "reader_acc", desc = "Reader's account")]
    #[account(
        1,
        writable,
        name = "researcher_profile_pda_acc",
        desc = "Researcher's profile PDA account"
    )]
    #[account(
        2,
        writable,
        name = "research_mint_collection_pda_acc",
        desc = "Research mint collection PDA account"
    )]
    #[account(
        3,
        writable,
        name = "paper_pda_acc",
        desc = "Research paper PDA account"
    )]
    #[account(
        4,
        writable,
        name = "fee_receiver_acc",
        desc = "Fee receiver's account"
    )]
    #[account(5, name = "system_program_acc", desc = "System program account")]
    MintResearchPaper(MintResearchPaper),
    #[account(
        0,
        writable,
        signer,
        name = "reputation_checker_acc",
        desc = "Reputation checker's account"
    )]
    #[account(
        1,
        writable,
        name = "researcher_profile_pda_acc",
        desc = "Researcher's profile account"
    )]
    CheckAndAssignReputation(CheckAndAssignReputation),
}

fn validate_create_researcher_profile_accounts(
    researcher_acc: &AccountInfo,
    researcher_profile_pda_acc: &AccountInfo,
) -> Result<(), DeResearcherError> {
    if !researcher_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner);
    }

    if !researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileAlreadyExists);
    }

    if !researcher_profile_pda_acc.is_writable {
        return Err(DeResearcherError::ImmutableAccount);
    }

    Ok(())
}

// Create a new researcher profile

pub fn create_researcher_profile_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: CreateResearcherProfile,
) -> ProgramResult {
    msg!("Instruction: CreateResearcherProfile");
    let accounts_iter = &mut accounts.iter();
    let researcher_acc = next_account_info(accounts_iter)?;
    let researcher_profile_pda_acc = next_account_info(accounts_iter)?;
    let system_program_acc = next_account_info(accounts_iter)?;

    let researcher_profile_pda = researcher_profile_pda_acc.key;

    let seeds: Vec<&[u8]> = vec![RESEARCHER_PROFILE_PDA_SEED, researcher_acc.key.as_ref()];

    validate_pda(seeds, researcher_profile_pda, data.pda_bump, program_id)?;

    validate_create_researcher_profile_accounts(researcher_acc, researcher_profile_pda_acc)?;

    let rent = Rent::get()?;

    let rent_exempt = rent.minimum_balance(ResearcherProfile::size());

    let create_researcher_profile_ix = system_instruction::create_account(
        researcher_acc.key,
        researcher_profile_pda_acc.key,
        rent_exempt,
        ResearcherProfile::size() as u64,
        program_id,
    );

    invoke_signed(
        &create_researcher_profile_ix,
        &[
            researcher_acc.clone(),
            researcher_profile_pda_acc.clone(),
            system_program_acc.clone(),
        ],
        &[&[
            RESEARCHER_PROFILE_PDA_SEED,
            researcher_acc.key.as_ref(),
            &[data.pda_bump],
        ]],
    )?;

    ResearcherProfile::create_new(researcher_profile_pda_acc, researcher_acc.key, data)?;

    Ok(())
}

fn validate_create_research_paper_accounts(
    publisher_acc: &AccountInfo,
    researcher_profile_pda_acc: &AccountInfo,
    paper_pda_acc: &AccountInfo,
) -> Result<(), DeResearcherError> {
    if !publisher_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner);
    }

    if researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileNotFound);
    }

    if !paper_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PaperAlreadyExists);
    }

    Ok(())
}

// Create a new research paper

pub fn create_research_paper_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: CreateResearchePaper,
) -> ProgramResult {
    msg!("Instruction: CreateResearchePaper");
    let accounts_iter = &mut accounts.iter();

    let publisher_acc = next_account_info(accounts_iter)?;

    let researcher_profile_pda_acc = next_account_info(accounts_iter)?;

    let paper_pda_acc = next_account_info(accounts_iter)?;

    let paper_pda = paper_pda_acc.key;

    let paper_seeds: Vec<&[u8]> = vec![
        RESEARCH_PAPER_PDA_SEED,
        data.paper_content_hash[..32].as_ref(),
        publisher_acc.key.as_ref(),
    ];

    validate_pda(paper_seeds, paper_pda, data.pda_bump, program_id)?;

    let researcher_profile_seeds = vec![RESEARCHER_PROFILE_PDA_SEED, publisher_acc.key.as_ref()];

    let researcher_profile_pda = researcher_profile_pda_acc.key;

    let researcher_profile =
        ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.data.borrow())?;

    validate_pda(
        researcher_profile_seeds,
        researcher_profile_pda,
        researcher_profile.bump,
        program_id,
    )?;

    validate_create_research_paper_accounts(
        publisher_acc,
        researcher_profile_pda_acc,
        paper_pda_acc,
    )?;

    let rent = Rent::get()?;

    let rent_exempt = rent.minimum_balance(ResearchPaper::size());

    let create_researche_paper_ix = system_instruction::create_account(
        publisher_acc.key,
        paper_pda_acc.key,
        rent_exempt,
        ResearchPaper::size() as u64,
        program_id,
    );

    let system_program_acc = next_account_info(accounts_iter)?;

    invoke_signed(
        &create_researche_paper_ix,
        &[
            publisher_acc.clone(),
            paper_pda_acc.clone(),
            system_program_acc.clone(),
        ],
        &[&[
            RESEARCH_PAPER_PDA_SEED,
            data.paper_content_hash[..32].as_ref(),
            publisher_acc.key.as_ref(),
            &[data.pda_bump],
        ]],
    )?;

    ResearchPaper::create_new(
        paper_pda_acc,
        researcher_profile_pda_acc,
        publisher_acc,
        data,
    )?;
    Ok(())
}

fn validate_publish_paper_accounts(
    publisher_acc: &AccountInfo,
    paper_pda_acc: &AccountInfo,
    paper_pda: &Pubkey,
) -> Result<(), DeResearcherError> {
    if !publisher_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner);
    }

    if paper_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PaperNotFound);
    }

    if paper_pda.ne(paper_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    Ok(())
}

// Publish a research paper

pub fn publish_paper_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: PublishPaper,
) -> ProgramResult {
    msg!("Instruction: PublishPaper");
    let accounts_iter = &mut accounts.iter();

    let publisher_acc = next_account_info(accounts_iter)?;

    let paper_pda_acc = next_account_info(accounts_iter)?;

    let paper_pda = paper_pda_acc.key;

    let paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

    let seeds: Vec<&[u8]> = vec![
        RESEARCH_PAPER_PDA_SEED,
        paper.paper_content_hash[..32].as_ref(),
        publisher_acc.key.as_ref(),
    ];

    validate_pda(seeds, paper_pda, data.pda_bump, program_id)?;
    validate_publish_paper_accounts(publisher_acc, paper_pda_acc, &paper_pda)?;

    ResearchPaper::publish_paper(paper_pda_acc, publisher_acc)?;

    Ok(())
}

fn validate_add_peer_review_accounts(
    reviewer_acc: &AccountInfo,
    researcher_profile_pda_acc: &AccountInfo,
    paper_pda_acc: &AccountInfo,
    peer_review_pda_acc: &AccountInfo,
) -> Result<(), DeResearcherError> {
    if !reviewer_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner);
    }

    if researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileNotFound);
    }

    if !peer_review_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PeerReviewAlreadyExists);
    }

    if paper_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PaperNotFound);
    }

    if !peer_review_pda_acc.is_writable {
        return Err(DeResearcherError::ImmutableAccount);
    }

    Ok(())
}

fn validate_researcher_for_peer_review(
    _researcher_profile: &ResearcherProfile,
) -> Result<(), DeResearcherError> {
    // TODO: add this check
    // if researcher_profile.state != ResearcherProfileState::Approved {
    //     return Err(DeResearcherError::NotAllowedForPeerReview);
    // }

    Ok(())
}

// Add a peer review to a research paper

pub fn add_peer_review_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: AddPeerReview,
) -> ProgramResult {
    msg!("Instruction: AddPeerReview");
    let accounts_iter = &mut accounts.iter();

    let reviewer_acc = next_account_info(accounts_iter)?;

    let researcher_profile_pda_acc = next_account_info(accounts_iter)?;

    let paper_pda_acc = next_account_info(accounts_iter)?;

    let peer_review_pda_acc = next_account_info(accounts_iter)?;

    let researcher_profile_pda = researcher_profile_pda_acc.key;

    let paper_pda = paper_pda_acc.key;

    let researcher_profile =
        ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.data.borrow())?;

    validate_researcher_for_peer_review(&researcher_profile)?;

    let peer_review_pda = peer_review_pda_acc.key;

    let peer_review_seeds = vec![
        PEER_REVIEW_PDA_SEED,
        paper_pda_acc.key.as_ref(),
        reviewer_acc.key.as_ref(),
    ];

    validate_pda(
        peer_review_seeds,
        peer_review_pda,
        data.pda_bump,
        program_id,
    )?;

    let paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

    if paper.creator_pubkey.eq(reviewer_acc.key) {
        return Err(DeResearcherError::PublisherCannotAddPeerReview.into());
    }

    let paper_seeds = vec![
        RESEARCH_PAPER_PDA_SEED,
        paper.paper_content_hash[..32].as_ref(),
        paper.creator_pubkey.as_ref(),
    ];

    validate_pda(paper_seeds, paper_pda, paper.bump, program_id)?;

    let researcher_profile_seeds = vec![RESEARCHER_PROFILE_PDA_SEED, reviewer_acc.key.as_ref()];

    let researcher_profile =
        ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.data.borrow())?;

    validate_pda(
        researcher_profile_seeds,
        researcher_profile_pda,
        researcher_profile.bump,
        program_id,
    )?;

    validate_add_peer_review_accounts(
        reviewer_acc,
        researcher_profile_pda_acc,
        paper_pda_acc,
        peer_review_pda_acc,
    )?;

    let rent = Rent::get()?;

    let rent_exempt = rent.minimum_balance(PeerReview::size());

    let create_peer_review_ix = system_instruction::create_account(
        reviewer_acc.key,
        peer_review_pda_acc.key,
        rent_exempt,
        PeerReview::size() as u64,
        program_id,
    );

    let system_program_acc = next_account_info(accounts_iter)?;

    invoke_signed(
        &create_peer_review_ix,
        &[
            reviewer_acc.clone(),
            peer_review_pda_acc.clone(),
            system_program_acc.clone(),
        ],
        &[&[
            PEER_REVIEW_PDA_SEED,
            paper_pda_acc.key.as_ref(),
            reviewer_acc.key.as_ref(),
            &[data.pda_bump],
        ]],
    )?;

    PeerReview::create_new(
        peer_review_pda_acc,
        reviewer_acc,
        paper_pda_acc,
        researcher_profile_pda_acc,
        data,
    )?;

    Ok(())
}

fn validate_mint_res_paper_accounts(
    reader_acc: &AccountInfo,
    researcher_profile_pda_acc: &AccountInfo,
    paper_pda_acc: &AccountInfo,
    fee_receiver_acc: &AccountInfo,
    paper: &ResearchPaper,
) -> Result<(), DeResearcherError> {
    if !reader_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner);
    }

    if researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileNotFound);
    }

    if paper_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PaperNotFound);
    }

    if fee_receiver_acc.key.ne(&paper.creator_pubkey) {
        return Err(DeResearcherError::InvalidFeeReceiver);
    }

    Ok(())
}

// Mint a research paper

pub fn mint_res_paper_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: MintResearchPaper,
) -> ProgramResult {
    msg!("Instruction: MintResearchPaper");
    let accounts_iter = &mut accounts.iter();

    let reader_acc = next_account_info(accounts_iter)?;

    let researcher_profile_pda_acc = next_account_info(accounts_iter)?;

    let research_mint_collection_pda_acc = next_account_info(accounts_iter)?;

    let paper_pda_acc = next_account_info(accounts_iter)?;

    let research_mint_collection_pda = research_mint_collection_pda_acc.key;

    let res_mint_collection_seeds =
        vec![RESEARCH_MINT_COLLECTION_PDA_SEED, reader_acc.key.as_ref()];

    validate_pda(
        res_mint_collection_seeds,
        research_mint_collection_pda,
        data.pda_bump,
        program_id,
    )?;

    let fee_receiver_acc = next_account_info(accounts_iter)?;

    let paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

    validate_mint_res_paper_accounts(
        reader_acc,
        researcher_profile_pda_acc,
        paper_pda_acc,
        fee_receiver_acc,
        &paper,
    )?;

    if research_mint_collection_pda_acc.data_is_empty() {
        let create_res_mint_collection_ix = system_instruction::create_account(
            reader_acc.key,
            research_mint_collection_pda_acc.key,
            Rent::get()?.minimum_balance(ResearchMintCollection::size() as usize),
            ResearchMintCollection::size() as u64,
            program_id,
        );

        let system_program_acc = next_account_info(accounts_iter)?;
        invoke_signed(
            &create_res_mint_collection_ix,
            &[
                reader_acc.clone(),
                research_mint_collection_pda_acc.clone(),
                system_program_acc.clone(),
            ],
            &[&[
                RESEARCH_MINT_COLLECTION_PDA_SEED,
                reader_acc.key.as_ref(),
                &[data.pda_bump],
            ]],
        )?;
    }

    if paper.access_fee > 0 {
        invoke(
            &system_instruction::transfer(
                reader_acc.key,
                &paper.creator_pubkey,
                paper.access_fee as u64,
            ),
            &[reader_acc.clone(), fee_receiver_acc.clone()],
        )?;
    }

    ResearchMintCollection::mint_paper(
        research_mint_collection_pda_acc,
        reader_acc,
        paper_pda_acc,
        researcher_profile_pda_acc,
        data,
    )?;

    Ok(())
}

pub fn validate_check_and_assign_reputation_accounts(
    reputation_checker_acc: &AccountInfo,
    researcher_profile_acc: &AccountInfo,
) -> Result<(), DeResearcherError> {
    if researcher_profile_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileNotFound);
    }

    if !researcher_profile_acc.is_writable {
        return Err(DeResearcherError::ImmutableAccount);
    }

    if reputation_checker_acc.key.ne(&REPUTATION_CHECKER_PUBKEY) {
        return Err(DeResearcherError::InvalidReputationChecker.into());
    }

    if !reputation_checker_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner);
    }

    Ok(())
}

// Check and assign reputation

pub fn check_and_assign_reputation_ix(
    accounts: &[AccountInfo],
    data: CheckAndAssignReputation,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let reputation_checker_acc = next_account_info(accounts_iter)?;

    let researcher_profile_pda_acc = next_account_info(accounts_iter)?;

    validate_check_and_assign_reputation_accounts(
        reputation_checker_acc,
        researcher_profile_pda_acc,
    )?;

    ResearcherProfile::assign_reputation(researcher_profile_pda_acc, data)?;

    Ok(())
}

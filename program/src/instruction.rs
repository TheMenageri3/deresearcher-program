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
        PeerReview, ResearchMintCollection, ResearchPaper, ResearcherProfile,
        ResearcherProfileState,
    },
};

pub const MIN_APPROVALS: u8 = 10;

const RESEARCH_PAPER_PDA_SEED: &[u8] = b"deres_paper";
const PEER_REVIEW_PDA_SEED: &[u8] = b"deres_review";

const RESEARCH_MINT_COLLECTION_PDA_SEED: &[u8] = b"deres_mint_collection";

const RESEARCHER_PROFILE_PDA_SEED: &[u8] = b"deres_profile";

const _MAX_REPUTATION: u8 = 100;

const _MIN_REPUTATION_FOR_PEER_REVIEW: u8 = 50;

pub const MAX_STRING_SIZE: usize = 64;

pub const RUST_STRING_ADDR_OFFSET: usize = 6;

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
        desc = "Reader's research mint collection PDA account"
    )]
    #[account(
        3,
        writable,
        name = "paper_pda_acc",
        desc = "Research paper PDA account"
    )]
    #[account(4, name = "system_program_acc", desc = "System program account")]
    #[account(
        5,
        writable,
        name = "fee_receiver_acc",
        desc = "Fee receiver's account"
    )]
    MintResearchPaper(MintResearchPaper),
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

    ResearcherProfile::create_new(researcher_profile_pda_acc, data)?;

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

    ResearchPaper::create_new(paper_pda_acc, researcher_profile_pda_acc, data)?;
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
    peer_review_pda: &Pubkey,
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

    if !paper_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PaperNotFound);
    }

    if peer_review_pda_acc.is_writable {
        return Err(DeResearcherError::ImmutableAccount);
    }

    if peer_review_pda.ne(peer_review_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    Ok(())
}

fn validate_researcher_for_peer_review(
    researcher_profile: &ResearcherProfile,
) -> Result<(), DeResearcherError> {
    if researcher_profile.state != ResearcherProfileState::Approved {
        return Err(DeResearcherError::NotAllowedForPeerReview);
    }

    Ok(())
}

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
        reviewer_acc.key.as_ref(),
        paper_pda_acc.key.as_ref(),
    ];

    validate_pda(
        peer_review_seeds,
        peer_review_pda,
        data.pda_bump,
        program_id,
    )?;

    let paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

    let paper_seeds = vec![
        RESEARCH_PAPER_PDA_SEED,
        paper.paper_content_hash[..32].as_ref(),
        paper.creator_pubkey.as_ref(),
    ];

    validate_pda(paper_seeds, paper_pda, data.pda_bump, program_id)?;

    let researcher_profile_seeds = vec![RESEARCHER_PROFILE_PDA_SEED, reviewer_acc.key.as_ref()];

    validate_pda(
        researcher_profile_seeds,
        researcher_profile_pda,
        data.pda_bump,
        program_id,
    )?;

    validate_add_peer_review_accounts(
        reviewer_acc,
        researcher_profile_pda_acc,
        paper_pda_acc,
        peer_review_pda_acc,
        &peer_review_pda,
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
            reviewer_acc.key.as_ref(),
            paper_pda_acc.key.as_ref(),
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
    research_mint_collection_pda_acc: &AccountInfo,
    paper_pda_acc: &AccountInfo,
    research_mint_collection_pda: &Pubkey,
    paper_pda: &Pubkey,
    fee_receiver_acc: &AccountInfo,
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

    if research_mint_collection_pda.ne(research_mint_collection_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    if paper_pda.ne(paper_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    if fee_receiver_acc.key.ne(&paper_pda_acc.key) {
        return Err(DeResearcherError::InvalidFeeReceiver);
    }

    Ok(())
}

pub fn mint_res_paper_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: MintResearchPaper,
) -> ProgramResult {
    msg!("Instruction: GetAccess");
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

    let paper_pda = paper_pda_acc.key;

    let paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

    let paper_seeds = vec![
        RESEARCH_PAPER_PDA_SEED,
        paper.paper_content_hash[..32].as_ref(),
        paper.creator_pubkey.as_ref(),
    ];

    validate_pda(paper_seeds, paper_pda, data.pda_bump, program_id)?;

    let fee_receiver_acc = next_account_info(accounts_iter)?;

    validate_mint_res_paper_accounts(
        reader_acc,
        researcher_profile_pda_acc,
        research_mint_collection_pda_acc,
        paper_pda_acc,
        &research_mint_collection_pda,
        &paper_pda,
        fee_receiver_acc,
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

    let paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

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

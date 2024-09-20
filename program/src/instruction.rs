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
        PaperState, PeerReview, ReaderWhitelist, ResearchPaper, ResearcherProfile,
        ResearcherProfileState, Review,
    },
};

const MIN_APPROVALS: u8 = 10;

const RESEARCH_PAPER_PDA_SEED: &[u8] = b"deres_research_paper";
const PEER_REVIEW_PDA_SEED: &[u8] = b"deres_peer_review";

const WHITELIST_PDA_SEED: &[u8] = b"deres_whitelist";

const RESEARCHER_PROFILE: &[u8] = b"deres_researcher_profile";

const _MAX_REPUTATION: u8 = 100;

const _MIN_REPUTATION_FOR_PEER_REVIEW: u8 = 50;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreateResearcherProfile {
    pub name: String,
    pub email: String,
    pub reputation: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreateResearchePaper {
    pub access_fee: u32,
    pub paper_content_hash: [u8; 64],
    pub meta_data_merkle_root: [u8; 64],
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddPeerReview {
    pub quality_of_research: u8,
    pub potential_for_real_world_use_case: u8,
    pub domain_knowledge: u8,
    pub practicality_of_result_obtained: u8,
    pub meta_data_merkle_root: [u8; 64],
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GetAccessToPaper {
    pub meta_data_merkle_root: [u8; 64],
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
    PublishPaper,
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
        name = "whitelist_pda_acc",
        desc = "Reader's whitelist PDA account"
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
    GetAccessToPaper(GetAccessToPaper),
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
    if !researcher_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner.into());
    }

    if !researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileAlreadyExists.into());
    }

    let rent = Rent::get()?;

    let system_program_acc = next_account_info(accounts_iter)?;

    let rent_exempt = rent.minimum_balance(ResearchPaper::size());

    let create_researcher_profile_ix = system_instruction::create_account(
        researcher_acc.key,
        researcher_profile_pda_acc.key,
        rent_exempt,
        ResearcherProfile::size() as u64,
        program_id,
    );

    let seeds: Vec<&[u8]> = vec![RESEARCHER_PROFILE, researcher_acc.key.as_ref()];

    let seeds_ref = seeds.as_ref();

    let (researcher_profile_pda, bump) = Pubkey::find_program_address(seeds_ref, program_id);

    if researcher_profile_pda.ne(researcher_profile_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    invoke_signed(
        &create_researcher_profile_ix,
        &[
            researcher_acc.clone(),
            researcher_profile_pda_acc.clone(),
            system_program_acc.clone(),
        ],
        &[seeds_ref, &[&[bump]]],
    )?;

    let researcher_profile = ResearcherProfile {
        address: *researcher_profile_pda_acc.key,
        name: data.name.as_bytes().try_into().unwrap(),
        state: ResearcherProfileState::AwaitingApproval,
        total_papers_published: 0,
        total_citations: 0,
        total_reviews: 0,
        reputation: data.reputation,
        meta_data_merkle_root: [0; 64],
    };

    let mut data_bytes: Vec<u8> = Vec::new();

    researcher_profile.serialize(&mut data_bytes)?;

    researcher_profile_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&data_bytes);

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

    if researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileNotFound.into());
    }

    let mut researcher_profile =
        ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.data.borrow())?;

    if !publisher_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner.into());
    }

    if !paper_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PaperAlreadyExists.into());
    }

    let rent = Rent::get()?;

    let rent_exempt = rent.minimum_balance(ResearchPaper::size());

    let seeds: Vec<&[u8]> = vec![b"deres_research_paper", publisher_acc.key.as_ref()];

    let seeds_ref = seeds.as_ref();

    let (paper_pda, bump) = Pubkey::find_program_address(seeds_ref, program_id);

    if paper_pda.ne(paper_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

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
        &[seeds_ref, &[&[bump]]],
    )?;

    let paper = ResearchPaper {
        address: *paper_pda_acc.key,
        state: PaperState::AwaitingPeerReview,
        creator_pubkey: *publisher_acc.key,
        access_fee: data.access_fee,
        version: 0,
        total_approvals: 0,
        total_citations: 0,
        paper_content_hash: data.paper_content_hash,
        meta_data_merkle_root: data.meta_data_merkle_root,
    };

    let mut paper_data_bytes: Vec<u8> = Vec::new();
    paper.serialize(&mut paper_data_bytes)?;
    paper_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&paper_data_bytes);

    researcher_profile.total_papers_published += 1;

    let mut profile_data_bytes: Vec<u8> = Vec::new();

    researcher_profile.serialize(&mut profile_data_bytes)?;

    researcher_profile_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&profile_data_bytes);
    Ok(())
}

pub fn publish_paper_ix(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Instruction: PublishPaper");
    let accounts_iter = &mut accounts.iter();

    let publisher_acc = next_account_info(accounts_iter)?;

    let paper_pda_acc = next_account_info(accounts_iter)?;

    let (paper_pda, _bump) = Pubkey::find_program_address(
        &[RESEARCH_PAPER_PDA_SEED, publisher_acc.key.as_ref()],
        program_id,
    );

    if paper_pda.ne(paper_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    if paper_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PaperNotFound.into());
    }

    let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

    if paper.creator_pubkey.ne(publisher_acc.key) || !publisher_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner.into());
    }

    if paper.state != PaperState::AwaitingPeerReview {
        return Err(DeResearcherError::InvalidState.into());
    }

    if paper.total_approvals < MIN_APPROVALS {
        return Err(DeResearcherError::NotEnoughApprovals.into());
    }

    paper.state = PaperState::Published;

    let mut data_bytes: Vec<u8> = Vec::new();

    paper.serialize(&mut data_bytes)?;

    paper_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&data_bytes);

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

    if researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileNotFound.into());
    }

    let mut researcher_profile =
        ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.data.borrow())?;

    if researcher_profile.state != ResearcherProfileState::Approved {
        return Err(DeResearcherError::NotAllowedForPeerReview.into());
    }

    if !peer_review_pda_acc.data_is_empty() {
        return Err(DeResearcherError::PeerReviewAlreadyExists.into());
    }

    if !reviewer_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner.into());
    }

    let (peer_review_pda, peer_review_bump) = Pubkey::find_program_address(
        &[
            PEER_REVIEW_PDA_SEED,
            reviewer_acc.key.as_ref(),
            peer_review_pda_acc.key.as_ref(),
        ],
        program_id,
    );

    let (paper_pda, _paper_bump) = Pubkey::find_program_address(
        &[RESEARCH_PAPER_PDA_SEED, paper_pda_acc.key.as_ref()],
        program_id,
    );

    if paper_pda.ne(paper_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    if peer_review_pda.ne(peer_review_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    let peer_review = PeerReview {
        address: peer_review_pda,
        reviewer_pubkey: *reviewer_acc.key,
        paper_pubkey: *paper_pda_acc.key,
        review: Review {
            quality_of_research: data.quality_of_research,
            potential_for_real_world_use_case: data.potential_for_real_world_use_case,
            domain_knowledge: data.domain_knowledge,
            practicality_of_result_obtained: data.practicality_of_result_obtained,
        },
        meta_data_merkle_root: data.meta_data_merkle_root,
    };

    let cumulative_score = peer_review.review.quality_of_research
        + peer_review.review.potential_for_real_world_use_case
        + peer_review.review.domain_knowledge
        + peer_review.review.practicality_of_result_obtained;

    let avg_score = cumulative_score / 4;

    if avg_score > 50 {
        let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

        paper.total_approvals += 1;
        paper.total_citations += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        paper.serialize(&mut data_bytes)?;

        paper_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);
    }

    let mut data_bytes: Vec<u8> = Vec::new();

    peer_review.serialize(&mut data_bytes)?;

    let rent = Rent::get()?;

    let rent_exempt = rent.minimum_balance(PeerReview::size());

    let create_peer_review_ix = system_instruction::create_account(
        reviewer_acc.key,
        peer_review_pda_acc.key,
        rent_exempt,
        PeerReview::size() as u64,
        program_id,
    );

    let seeds = vec![
        PEER_REVIEW_PDA_SEED,
        reviewer_acc.key.as_ref(),
        paper_pda_acc.key.as_ref(),
    ];
    let seeds_ref = seeds.as_ref();

    let system_program_acc = next_account_info(accounts_iter)?;

    invoke_signed(
        &create_peer_review_ix,
        &[
            reviewer_acc.clone(),
            peer_review_pda_acc.clone(),
            system_program_acc.clone(),
        ],
        &[seeds_ref, &[&[peer_review_bump]]],
    )?;

    peer_review_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&data_bytes);

    researcher_profile.total_reviews += 1;

    let mut profile_data_bytes: Vec<u8> = Vec::new();

    researcher_profile.serialize(&mut profile_data_bytes)?;

    researcher_profile_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&profile_data_bytes);

    Ok(())
}

pub fn get_access_ix(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: GetAccessToPaper,
) -> ProgramResult {
    msg!("Instruction: GetAccess");
    let accounts_iter = &mut accounts.iter();

    let reader_acc = next_account_info(accounts_iter)?;

    if !reader_acc.is_signer {
        return Err(DeResearcherError::InvalidSigner.into());
    }

    let researcher_profile_pda_acc = next_account_info(accounts_iter)?;

    if researcher_profile_pda_acc.data_is_empty() {
        return Err(DeResearcherError::ResearcherProfileNotFound.into());
    }

    let whitelist_pda_acc = next_account_info(accounts_iter)?;

    let paper_pda_acc = next_account_info(accounts_iter)?;

    let (whitelist_pda, whitelist_bump) =
        Pubkey::find_program_address(&[WHITELIST_PDA_SEED, reader_acc.key.as_ref()], program_id);

    if whitelist_pda.ne(whitelist_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    let (paper_pda, _paper_bump) = Pubkey::find_program_address(
        &[RESEARCH_PAPER_PDA_SEED, paper_pda_acc.key.as_ref()],
        program_id,
    );

    if paper_pda.ne(paper_pda_acc.key) {
        return Err(DeResearcherError::PubkeyMismatch.into());
    }

    if whitelist_pda_acc.data_is_empty() {
        let create_whitelist_ix = system_instruction::create_account(
            reader_acc.key,
            whitelist_pda_acc.key,
            Rent::get()?.minimum_balance(ReaderWhitelist::size() as usize),
            ReaderWhitelist::size() as u64,
            program_id,
        );

        let seeds = vec![WHITELIST_PDA_SEED, reader_acc.key.as_ref()];
        let seeds_ref = seeds.as_ref();

        let system_program_acc = next_account_info(accounts_iter)?;
        invoke_signed(
            &create_whitelist_ix,
            &[
                reader_acc.clone(),
                whitelist_pda_acc.clone(),
                system_program_acc.clone(),
            ],
            &[seeds_ref, &[&[whitelist_bump]]],
        )?;
    }

    let fee_receiver_acc = next_account_info(accounts_iter)?;

    let mut researcher_profile =
        ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.data.borrow())?;

    let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

    if paper.creator_pubkey.ne(fee_receiver_acc.key) {
        return Err(DeResearcherError::InvalidFeeReceiver.into());
    }

    invoke(
        &system_instruction::transfer(
            reader_acc.key,
            &paper.creator_pubkey,
            paper.access_fee as u64,
        ),
        &[reader_acc.clone(), fee_receiver_acc.clone()],
    )?;

    let whitelist = ReaderWhitelist {
        reader_pubkey: *reader_acc.key,
        data_merkle_root: data.meta_data_merkle_root,
    };

    let mut whitelist_data_bytes: Vec<u8> = Vec::new();

    whitelist.serialize(&mut whitelist_data_bytes)?;

    whitelist_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&whitelist_data_bytes);

    paper.total_citations += 1;

    let mut paper_data_bytes: Vec<u8> = Vec::new();

    paper.serialize(&mut paper_data_bytes)?;

    paper_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&paper_data_bytes);

    researcher_profile.total_citations += 1;

    let mut profile_data_bytes: Vec<u8> = Vec::new();

    researcher_profile.serialize(&mut profile_data_bytes)?;

    researcher_profile_pda_acc
        .try_borrow_mut_data()?
        .copy_from_slice(&profile_data_bytes);

    Ok(())
}

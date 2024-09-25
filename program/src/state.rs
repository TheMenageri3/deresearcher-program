use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    error::DeResearcherError,
    instruction::{
        AddPeerReview, CheckAndAssignReputation, CreateResearchePaper, CreateResearcherProfile,
        MintResearchPaper, MAX_REPUTATION, MAX_STRING_SIZE, MIN_APPROVALS_FOR_PUBLISH,
        MIN_REPUTATION_FOR_PEER_REVIEW,
    },
};

pub fn checked_string_convt_to_64_bytes(
    data: &str,
) -> Result<[u8; MAX_STRING_SIZE], DeResearcherError> {
    if data.len() > MAX_STRING_SIZE {
        return Err(DeResearcherError::SizeOverflow);
    }

    let mut data_bytes: [u8; MAX_STRING_SIZE] = [0; MAX_STRING_SIZE];

    let end = data.len();

    data_bytes[..end].copy_from_slice(data.as_bytes().as_ref());

    Ok(data_bytes)
}

#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PaperState {
    AwaitingPeerReview,
    InPeerReview,
    ApprovedToPublish,
    RequiresRevision,
    Published,
    Minted,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, PartialOrd)]
pub enum ResearcherProfileState {
    AwaitingApproval,
    Approved,
    Rejected,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, ShankAccount)]
pub struct ResearcherProfile {
    pub address: Pubkey,                 // Researcher pda pubkey key 32 bytes
    pub researcher_pubkey: Pubkey,       // Researcher's public key 32 bytes
    pub name: [u8; 64],                  // Researcher's name 32 bytes
    pub state: ResearcherProfileState,   // Current state of the researcher 1 byte
    pub total_papers_published: u64,     // Total papers published 8 bytes
    pub total_citations: u64,            // Total citations 8 bytes
    pub total_reviews: u64,              // Total reviews 8 bytes
    pub reputation: u8,                  // Reputation score 1 bytes (out of 100)
    pub meta_data_merkle_root: [u8; 64], // Data merkle root 64 bytes
    pub bump: u8,                        // Bump seed 1 byte
}

impl ResearcherProfile {
    pub fn size() -> usize {
        32 + 32 + 64 + 1 + 8 + 8 + 8 + 1 + 64 + 1 // 211
    }

    pub fn create_new(
        researcher_profile_pda_acc: &AccountInfo,
        researcher_pubkey: &Pubkey,
        data: CreateResearcherProfile,
    ) -> ProgramResult {
        let name_bytes = checked_string_convt_to_64_bytes(&data.name)?;
        let researcher_profile = Self {
            address: *researcher_profile_pda_acc.key,
            researcher_pubkey: *researcher_pubkey,
            name: name_bytes,
            state: ResearcherProfileState::AwaitingApproval,
            total_papers_published: 0,
            total_citations: 0,
            total_reviews: 0,
            reputation: 0,
            meta_data_merkle_root: [0; 64],
            bump: data.pda_bump,
        };

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        Ok(())
    }

    pub fn assign_reputation(
        researcher_profile_pda_acc: &AccountInfo,
        data: CheckAndAssignReputation,
    ) -> ProgramResult {
        let mut researcher_profile =
            ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.try_borrow_data()?)?;

        researcher_profile.reputation = data.reputation;

        if data.reputation > MAX_REPUTATION {
            return Err(DeResearcherError::SizeOverflow.into());
        }

        if data.reputation > MIN_REPUTATION_FOR_PEER_REVIEW {
            researcher_profile.state = ResearcherProfileState::Approved;
        } else {
            researcher_profile.state = ResearcherProfileState::Rejected;
        }

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        Ok(())
    }
}

// pub struct MetaData {
//     pub title: String,                      // Title of the research paper
//     pub authors: Vec<(String, Pubkey)>,     // Authors and their public keys
//     pub creation_date_timestamp: u64,       // Creation date as a timestamp
//     pub domain: String,                     // Domain of the research
//     pub abstract_data: String,              // Abstract text
//     pub decentralized_storage_link: String, // Link to the PDF in decentralized storage
// }

#[derive(Debug, BorshDeserialize, BorshSerialize, ShankAccount)]
pub struct ResearchPaper {
    pub address: Pubkey,                 // Paper's public key 32 bytes
    pub creator_pubkey: Pubkey,          // Creator's public key 32 bytes
    pub state: PaperState,               // Current state of the paper 1 byte
    pub access_fee: u32,                 // Access fee for the paper 4 bytes
    pub version: u8,                     // Version of the paper 1 byte
    pub paper_content_hash: [u8; 64],    // Hash of the paper's content 64 bytes
    pub total_approvals: u8,             // Total approvals 1 byte
    pub total_citations: u64,            // Total citations 8 bytes
    pub total_mints: u64,                // Total mints 8 bytes
    pub meta_data_merkle_root: [u8; 64], // Data merkle root 64 bytes
    pub bump: u8,                        // Bump seed 1 byte
}

impl ResearchPaper {
    pub fn size() -> usize {
        32 + 32 + 1 + 4 + 1 + 64 + 1 + 8 + 8 + 64 + 1 //208
    }

    pub fn create_new(
        research_paper_pda_acc: &AccountInfo,
        researcher_profile_pda_acc: &AccountInfo,
        publisher_acc: &AccountInfo,
        data: CreateResearchePaper,
    ) -> ProgramResult {
        let content_hash_bytes = checked_string_convt_to_64_bytes(&data.paper_content_hash)?;

        let merkle_root_bytes = checked_string_convt_to_64_bytes(&data.meta_data_merkle_root)?;

        let research_paper = Self {
            address: *research_paper_pda_acc.key,
            creator_pubkey: *publisher_acc.key,
            state: PaperState::AwaitingPeerReview,
            access_fee: data.access_fee,
            version: 0,
            paper_content_hash: content_hash_bytes,
            total_approvals: 0,
            total_citations: 0,
            total_mints: 0,
            meta_data_merkle_root: merkle_root_bytes,
            bump: data.pda_bump,
        };

        let mut data_bytes: Vec<u8> = Vec::new();

        research_paper.serialize(&mut data_bytes)?;

        research_paper_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        let mut researcher_profile =
            ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.try_borrow_data()?)?;

        researcher_profile.total_papers_published += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        Ok(())
    }

    pub fn publish_paper(
        paper_pda_acc: &AccountInfo,
        publisher_acc: &AccountInfo,
    ) -> ProgramResult {
        let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.try_borrow_data()?)?;

        if paper.creator_pubkey.ne(publisher_acc.key) {
            return Err(DeResearcherError::PubkeyMismatch.into());
        }

        if paper.state != PaperState::ApprovedToPublish {
            return Err(DeResearcherError::InvalidState.into());
        }

        paper.state = PaperState::Published;

        let mut data_bytes: Vec<u8> = Vec::new();

        paper.serialize(&mut data_bytes)?;

        paper_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        Ok(())
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize, ShankAccount)]
pub struct PeerReview {
    pub address: Pubkey,         // Peer Review Entry's public key 32 bytes
    pub reviewer_pubkey: Pubkey, // Reviewer's public key 32 bytes
    pub paper_pubkey: Pubkey,    // Paper's public key 32 bytes
    pub quality_of_research: u8, // Rating for quality of research (out of 100)
    pub potential_for_real_world_use_case: u8, // Rating for potential real-world use case (out of 100)
    pub domain_knowledge: u8,                  // Rating for domain knowledge (out of 100)
    pub practicality_of_result_obtained: u8,   // Rating for practicality of the result (out of 100)
    pub meta_data_merkle_root: [u8; 64],       // Data merkle root 64 bytes
    pub bump: u8,                              // Bump seed 1 byte
}

impl PeerReview {
    pub fn size() -> usize {
        32 + 32 + 32 + 1 + 1 + 1 + 1 + 64 + 1 //165
    }

    pub fn create_new(
        peer_review_pda_acc: &AccountInfo,
        reviewer_acc: &AccountInfo,
        paper_pda_acc: &AccountInfo,
        researcher_profile_pda_acc: &AccountInfo,
        data: AddPeerReview,
    ) -> ProgramResult {
        let merkle_root_bytes = checked_string_convt_to_64_bytes(&data.meta_data_merkle_root)?;

        let peer_review = Self {
            address: *peer_review_pda_acc.key,
            reviewer_pubkey: *reviewer_acc.key,
            paper_pubkey: *paper_pda_acc.key,
            quality_of_research: data.quality_of_research,
            potential_for_real_world_use_case: data.potential_for_real_world_use_case,
            domain_knowledge: data.domain_knowledge,
            practicality_of_result_obtained: data.practicality_of_result_obtained,
            meta_data_merkle_root: merkle_root_bytes,
            bump: data.pda_bump,
        };

        let cumulative_score = peer_review.quality_of_research as u16
            + peer_review.potential_for_real_world_use_case as u16
            + peer_review.domain_knowledge as u16
            + peer_review.practicality_of_result_obtained as u16;

        let avg_score = cumulative_score / 4;
        let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

        if paper.state == PaperState::AwaitingPeerReview {
            paper.state = PaperState::InPeerReview;
        }

        if avg_score > 50 {
            paper.total_approvals += 1;
        }

        if paper.total_approvals >= MIN_APPROVALS_FOR_PUBLISH {
            paper.state = PaperState::ApprovedToPublish;
        }

        paper.total_citations += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        paper.serialize(&mut data_bytes)?;

        paper_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        let mut data_bytes: Vec<u8> = Vec::new();

        peer_review.serialize(&mut data_bytes)?;

        peer_review_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        let mut researcher_profile =
            ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.try_borrow_data()?)?;

        researcher_profile.total_reviews += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        Ok(())
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize, ShankAccount)]
pub struct ResearchMintCollection {
    pub reader_pubkey: Pubkey,      // Reader's public key 32 bytes
    pub data_merkle_root: [u8; 64], // Data merkle root 64 bytes
    pub bump: u8,                   // Bump seed 1 byte
}

impl ResearchMintCollection {
    pub fn size() -> usize {
        32 + 64 + 1 //97
    }

    pub fn mint_paper(
        research_mint_collection_pda_acc: &AccountInfo,
        reader_acc: &AccountInfo,
        paper_pda_acc: &AccountInfo,
        researcher_profile_pda_acc: &AccountInfo,
        data: MintResearchPaper,
    ) -> ProgramResult {
        let merkle_root_bytes = checked_string_convt_to_64_bytes(&data.meta_data_merkle_root)?;

        let research_mint_collection = Self {
            reader_pubkey: *reader_acc.key,
            data_merkle_root: merkle_root_bytes,
            bump: data.pda_bump,
        };

        let mut data_bytes: Vec<u8> = Vec::new();

        research_mint_collection.serialize(&mut data_bytes)?;

        research_mint_collection_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

        if paper.state != PaperState::Published {
            return Err(DeResearcherError::InvalidState.into());
        }

        paper.total_citations += 1;

        paper.total_mints += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        paper.serialize(&mut data_bytes)?;

        paper_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        let mut researcher_profile =
            ResearcherProfile::try_from_slice(&researcher_profile_pda_acc.try_borrow_data()?)?;

        researcher_profile.total_citations += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        Ok(())
    }
}

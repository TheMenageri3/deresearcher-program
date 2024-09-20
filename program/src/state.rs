use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    error::DeResearcherError,
    instruction::{
        AddPeerReview, CreateResearchePaper, CreateResearcherProfile, GetAccessToPaper,
        ACCCOUNTS_DATA_OFFSET, MAX_STRING_SIZE, MIN_APPROVALS, RUST_STRING_ADDR_OFFSET,
    },
};

#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PaperState {
    AwaitingPeerReview,
    InPeerReview,
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
    pub address: Pubkey,                 // Researcher's public key 32 bytes
    pub name: [u8; 32],                  // Researcher's name 32 bytes
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
        std::mem::size_of::<Self>()
    }

    pub fn create_new(
        researcher_profile_pda_acc: &AccountInfo,
        data: CreateResearcherProfile,
    ) -> ProgramResult {
        if data.name.len() > MAX_STRING_SIZE {
            return Err(DeResearcherError::SizeOverflow.into());
        }

        let mut name_bytes: [u8; MAX_STRING_SIZE] = [0; MAX_STRING_SIZE];

        name_bytes[..data.name.len()]
            .copy_from_slice(&data.name.as_bytes()[RUST_STRING_ADDR_OFFSET..MAX_STRING_SIZE]);

        let researcher_profile = Self {
            address: *researcher_profile_pda_acc.key,
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

        researcher_profile_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..]
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
    pub paper_content_hash: [u8; 64],    // Hash of the paper's content 32 bytes
    pub total_approvals: u8,             // Total approvals 1 byte
    pub total_citations: u64,            // Total citations 8 bytes
    pub meta_data_merkle_root: [u8; 64], // Data merkle root 64 bytes
    pub bump: u8,                        // Bump seed 1 byte
}

impl ResearchPaper {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn create_new(
        research_paper_pda_acc: &AccountInfo,
        researcher_profile_pda_acc: &AccountInfo,
        data: CreateResearchePaper,
    ) -> ProgramResult {
        let research_paper = Self {
            address: *research_paper_pda_acc.key,
            creator_pubkey: *researcher_profile_pda_acc.key,
            state: PaperState::AwaitingPeerReview,
            access_fee: data.access_fee,
            version: 0,
            paper_content_hash: data.paper_content_hash,
            total_approvals: 0,
            total_citations: 0,
            meta_data_merkle_root: data.meta_data_merkle_root,
            bump: data.pda_bump,
        };

        let mut data_bytes: Vec<u8> = Vec::new();

        research_paper.serialize(&mut data_bytes)?;

        research_paper_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..]
            .copy_from_slice(&data_bytes);

        let mut researcher_profile = ResearcherProfile::try_from_slice(
            &researcher_profile_pda_acc.try_borrow_data()?[ACCCOUNTS_DATA_OFFSET..],
        )?;

        researcher_profile.total_papers_published += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..]
            .copy_from_slice(&data_bytes);

        Ok(())
    }

    pub fn publish_paper(
        paper_pda_acc: &AccountInfo,
        publisher_acc: &AccountInfo,
    ) -> ProgramResult {
        let mut paper = ResearchPaper::try_from_slice(
            &paper_pda_acc.try_borrow_data()?[ACCCOUNTS_DATA_OFFSET..],
        )?;

        if paper.state != PaperState::AwaitingPeerReview {
            return Err(DeResearcherError::InvalidState.into());
        }

        if paper.total_approvals < MIN_APPROVALS {
            return Err(DeResearcherError::NotEnoughApprovals.into());
        }

        if paper.creator_pubkey.ne(publisher_acc.key) {
            return Err(DeResearcherError::PubkeyMismatch.into());
        }

        paper.state = PaperState::Published;

        let mut data_bytes: Vec<u8> = Vec::new();

        paper.serialize(&mut data_bytes)?;

        paper_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..].copy_from_slice(&data_bytes);

        Ok(())
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Review {
    pub quality_of_research: u8, // Rating for quality of research (out of 100)
    pub potential_for_real_world_use_case: u8, // Rating for potential real-world use case (out of 100)
    pub domain_knowledge: u8,                  // Rating for domain knowledge (out of 100)
    pub practicality_of_result_obtained: u8, // Rating for practicality of the result (out of 100)             // Comments from the peer reviewer
}

// pub struct MetaData{
//   review_comments:String
//}

#[derive(Debug, BorshDeserialize, BorshSerialize, ShankAccount)]
pub struct PeerReview {
    pub address: Pubkey,                 // Peer Review Entry's public key 32 bytes
    pub reviewer_pubkey: Pubkey,         // Reviewer's public key 32 bytes
    pub paper_pubkey: Pubkey,            // Paper's public key 32 bytes
    pub review: Review,                  // Review 1 byte
    pub meta_data_merkle_root: [u8; 64], // Data merkle root 64 bytes
    pub bump: u8,                        // Bump seed 1 byte
}

impl PeerReview {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn create_new(
        peer_review_pda_acc: &AccountInfo,
        reviewer_acc: &AccountInfo,
        paper_pda_acc: &AccountInfo,
        researcher_profile_pda_acc: &AccountInfo,
        data: AddPeerReview,
    ) -> ProgramResult {
        let peer_review = Self {
            address: *peer_review_pda_acc.key,
            reviewer_pubkey: *reviewer_acc.key,
            paper_pubkey: *paper_pda_acc.key,
            review: Review {
                quality_of_research: data.quality_of_research,
                potential_for_real_world_use_case: data.potential_for_real_world_use_case,
                domain_knowledge: data.domain_knowledge,
                practicality_of_result_obtained: data.practicality_of_result_obtained,
            },
            meta_data_merkle_root: data.meta_data_merkle_root,
            bump: data.pda_bump,
        };

        let cumulative_score = peer_review.review.quality_of_research
            + peer_review.review.potential_for_real_world_use_case
            + peer_review.review.domain_knowledge
            + peer_review.review.practicality_of_result_obtained;

        let avg_score = cumulative_score / 4;
        let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

        if avg_score > 50 {
            paper.total_approvals += 1;
        }

        paper.total_citations += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        paper.serialize(&mut data_bytes)?;

        paper_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        let mut data_bytes: Vec<u8> = Vec::new();

        peer_review.serialize(&mut data_bytes)?;

        peer_review_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..]
            .copy_from_slice(&data_bytes);

        let mut researcher_profile = ResearcherProfile::try_from_slice(
            &researcher_profile_pda_acc.try_borrow_data()?[ACCCOUNTS_DATA_OFFSET..],
        )?;

        researcher_profile.total_reviews += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..]
            .copy_from_slice(&data_bytes);

        Ok(())
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize, ShankAccount)]
pub struct ReaderWhitelist {
    pub reader_pubkey: Pubkey,      // Reader's public key 32 bytes
    pub data_merkle_root: [u8; 64], // Data merkle root 64 bytes
    pub bump: u8,                   // Bump seed 1 byte
}

impl ReaderWhitelist {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn access_paper(
        whitelist_pda_acc: &AccountInfo,
        reader_acc: &AccountInfo,
        paper_pda_acc: &AccountInfo,
        researcher_profile_pda_acc: &AccountInfo,
        data: GetAccessToPaper,
    ) -> ProgramResult {
        let reader_whitelist = Self {
            reader_pubkey: *reader_acc.key,
            data_merkle_root: data.meta_data_merkle_root,
            bump: data.pda_bump,
        };

        let mut data_bytes: Vec<u8> = Vec::new();

        reader_whitelist.serialize(&mut data_bytes)?;

        whitelist_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..]
            .copy_from_slice(&data_bytes);

        let mut paper = ResearchPaper::try_from_slice(&paper_pda_acc.data.borrow())?;

        if paper.state != PaperState::Published {
            return Err(DeResearcherError::InvalidState.into());
        }

        paper.total_citations += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        paper.serialize(&mut data_bytes)?;

        paper_pda_acc
            .try_borrow_mut_data()?
            .copy_from_slice(&data_bytes);

        let mut researcher_profile = ResearcherProfile::try_from_slice(
            &researcher_profile_pda_acc.try_borrow_data()?[ACCCOUNTS_DATA_OFFSET..],
        )?;

        researcher_profile.total_citations += 1;

        let mut data_bytes: Vec<u8> = Vec::new();

        researcher_profile.serialize(&mut data_bytes)?;

        researcher_profile_pda_acc.try_borrow_mut_data()?[ACCCOUNTS_DATA_OFFSET..]
            .copy_from_slice(&data_bytes);

        Ok(())
    }
}

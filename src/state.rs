use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshDeserialize, BorshSerialize, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum PaperState {
    AwaitingPeerReview,
    InPeerReview,
    RequiresRevision,
    Published,
    Minted,
}

// pub struct MetaData {
//     pub title: String,                      // Title of the research paper
//     pub authors: Vec<(String, Pubkey)>,     // Authors and their public keys
//     pub creation_date_timestamp: u64,       // Creation date as a timestamp
//     pub domain: String,                     // Domain of the research
//     pub abstract_data: String,              // Abstract text
//     pub decentralized_storage_link: String, // Link to the PDF in decentralized storage
// }

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct ResearchPaper {
    pub address: Pubkey,                 // Paper's public key 32 bytes
    pub creator_pubkey: Pubkey,          // Creator's public key 32 bytes
    pub state: PaperState,               // Current state of the paper 1 byte
    pub access_fee: u32,                 // Access fee for the paper 4 bytes
    pub version: u8,                     // Version of the paper 1 byte
    pub paper_content_hash: [u8; 64],    // Hash of the paper's content 32 bytes
    pub totoal_approvals: u8,            // Total approvals 1 byte
    pub meta_data_merkle_root: [u8; 64], // Data merkle root 64 bytes
}

impl ResearchPaper {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
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

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct PeerReview {
    pub address: Pubkey,                 // Peer Review Entry's public key 32 bytes
    pub reviewer_pubkey: Pubkey,         // Reviewer's public key 32 bytes
    pub paper_pubkey: Pubkey,            // Paper's public key 32 bytes
    pub review: Review,                  // Review 1 byte
    pub meta_data_merkle_root: [u8; 64], // Data merkle root 64 bytes
}

impl PeerReview {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct ReaderWhitelist {
    pub reader_pubkey: Pubkey,      // Reader's public key 32 bytes
    pub data_merkle_root: [u8; 64], // Data merkle root 64 bytes
}

impl ReaderWhitelist {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

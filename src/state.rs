use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

// pub enum PaperState {
//     AwaitingPeerReview,
//     InPeerReview,
//     RequiresRevision,
//     Published,
//     Minted,
// }

// pub struct ResearchPaper {
//     pub paper_pubkey: Pubkey,               // Paper's public key
//     pub creator_pubkey: Pubkey,             // Creator's public key
//     pub state: PaperState,                  // Current state of the paper
//     pub title: String,                      // Title of the research paper
//     pub authors: Vec<(String, Pubkey)>,     // Authors and their public keys
//     pub creation_date_timestamp: u64,       // Creation date as a timestamp
//     pub domain: String,                     // Domain of the research
//     pub abstract_data: String,              // Abstract text
//     pub peer_approval_count: u32,           // Number of peer approvals
//     pub access_fee: u32,                    // Access fee for the paper
//     pub version: u8,                        // Version of the paper
//     pub paper_content_hash: String,         // Hash of the paper's content
//     pub decentralized_storage_link: String, // Link to the PDF in decentralized storage
// }

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct ResearchPaper {
    paper_pubkey: Pubkey,       // Paper's public key 32 bytes
    data_merkle_root: [u8; 64], // Data merkle root 64 bytes
}

impl ResearchPaper {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

// pub struct Review {
//     pub peer_pubkey: Pubkey,                         // Peer reviewer's public key
//     pub quality_of_research: u8,                     // Rating for quality of research (out of 8)
//     pub potential_for_real_world_use_case: u8,       // Rating for potential real-world use case (out of 8)
//     pub domain_knowledge: u8,                        // Rating for domain knowledge (out of 8)
//     pub practicality_of_result_obtained: u8,         // Rating for practicality of the result (out of 8)
//     pub review_comments: String,                     // Comments from the peer reviewer
// }

// pub struct PeerReviews {
//     pub paper_pubkey: Pubkey,                        // Public key of the paper being reviewed
//     pub reviews: Vec<Review>,                        // A list of reviews associated with the paper
// }

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct PeerReview {
    peer_review_entry_pubkey: Pubkey, // Peer Review Entry's public key 32 bytes
    paper_pubkey: Pubkey,             // Paper's public key 32 bytes
    data_merkle_root: [u8; 64],       // Data merkle root 64 bytes
}

impl PeerReview {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct ReaderWhitelist {
    reader_pubkey: Pubkey,      // Reader's public key 32 bytes
    data_merkle_root: [u8; 64], // Data merkle root 64 bytes
}

impl ReaderWhitelist {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

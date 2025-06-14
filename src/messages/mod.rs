#![allow(async_fn_in_trait)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use frostgate_zkip::types::ProofMetadata;

/// Supported chain identifiers. Extend as needed for more chains.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum ChainId {
    /// Ethereum blockchain
    Ethereum,
    /// Polkadot blockchain
    Polkadot,
    /// Solana blockchain
    Solana,
    /// Unknown or unsupported chain
    #[serde(other)]
    Unknown,
}

impl ChainId {
    /// Convert chain ID to u64 for serialization
    pub fn to_u64(&self) -> u64 {
        match self {
            ChainId::Ethereum => 0,
            ChainId::Polkadot => 1,
            ChainId::Solana => 2,
            ChainId::Unknown => u64::MAX,
        }
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            ChainId::Ethereum => "Ethereum",
            ChainId::Polkadot => "Polkadot",
            ChainId::Solana => "Solana",
            ChainId::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

impl std::convert::TryFrom<u64> for ChainId {
    type Error = ();

    /// Attempts to convert a u64 into a ChainId.
    /// 
    /// # Errors
    /// Returns `Ok(ChainId::Unknown)` for unrecognized chain IDs.
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ChainId::Ethereum),
            1 => Ok(ChainId::Polkadot),
            2 => Ok(ChainId::Solana),
            _ => Ok(ChainId::Unknown),
        }
    }
}

/// A zero-knowledge proof with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// The actual proof bytes
    pub data: Vec<u8>,
    /// Metadata about the proof
    pub metadata: ProofMetadata,
}

/// The canonical cross-chain message structure for Frostgate.
///
/// Includes all data necessary for verification and replay protection.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrostMessage {
    /// Unique message ID (UUID v4 for global uniqueness).
    pub id: Uuid,
    /// Source chain identifier.
    pub from_chain: ChainId,
    /// Destination chain identifier.
    pub to_chain: ChainId,
    /// Arbitrary user/application payload (should be encoded as required).
    pub payload: Vec<u8>,
    /// Zero-knowledge proof attached to the message (optional for some flows).
    pub proof: Option<Proof>,
    /// Unix timestamp (seconds) for message creation.
    pub timestamp: u64,
    /// Per-sender nonce for replay protection.
    pub nonce: u64,
    /// Optional cryptographic signature (by relayer/operator, not always required).
    pub signature: Option<Vec<u8>>,
    /// Optional relayer or protocol fee (in smallest unit of source chain).
    pub fee: Option<u128>,
    /// Extensible metadata for debugging, audit, or protocol extensions.
    pub metadata: Option<HashMap<String, String>>,
}

impl FrostMessage {
    /// Construct a new unsigned FrostMessage.
    ///
    /// # Arguments
    /// * `from_chain` - Source chain identifier
    /// * `to_chain` - Destination chain identifier
    /// * `payload` - Message payload as bytes
    /// * `nonce` - Per-sender nonce for replay protection
    /// * `timestamp` - Unix timestamp in seconds
    pub fn new(
        from_chain: ChainId,
        to_chain: ChainId,
        payload: Vec<u8>,
        nonce: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_chain,
            to_chain,
            payload,
            proof: None,
            timestamp,
            nonce,
            signature: None,
            fee: None,
            metadata: None,
        }
    }
}

/// A trait for messages that can be sent across chains.
pub trait CrossChainMessage {
    /// Get the unique identifier of the message.
    fn id(&self) -> Uuid;
    /// Get the message payload as a byte slice.
    fn payload(&self) -> &[u8];
    /// Get any chain-specific data associated with the message.
    fn chain_specific_data(&self) -> Option<&[u8]>;
}

impl CrossChainMessage for FrostMessage {
    fn id(&self) -> Uuid {
        self.id
    }

    fn payload(&self) -> &[u8] {
        &self.payload
    }

    fn chain_specific_data(&self) -> Option<&[u8]> {
        None // FrostMessage uses metadata HashMap for chain-specific data
    }
}

/// Message status for querying relay pipeline progress.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum MessageStatus {
    /// Message is pending processing
    Pending,
    /// Message is being processed
    InFlight,
    /// Message has been confirmed
    Confirmed,
    /// Message processing failed with error
    Failed(String),
}

/// Transaction hash or equivalent per chain.
/// 
/// This type represents a transaction identifier which may vary in format
/// depending on the specific blockchain (e.g., 32 bytes for Ethereum,
/// different formats for other chains).
pub type TxHash = Vec<u8>;

/// Message event structure (from source chain).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageEvent {
    /// The message associated with this event
    pub message: FrostMessage,
    /// Optional transaction hash if available
    pub tx_hash: Option<TxHash>,
    /// Optional block number where the event was emitted
    pub block_number: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frost_message_basic() {
        let msg = FrostMessage::new(
            ChainId::Ethereum,
            ChainId::Solana,
            b"test-payload".to_vec(),
            1,
            1_725_000_000,
        );
        let ser = serde_json::to_string(&msg).unwrap();
        let de: FrostMessage = serde_json::from_str(&ser).unwrap();
        assert_eq!(msg.from_chain, de.from_chain);
        assert_eq!(msg.payload, de.payload);
    }
}
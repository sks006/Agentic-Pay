// Solana Pay transaction-request construction will live here.

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use bincode::Options;
use solana_sdk::transaction::VersionedTransaction;
use crate::error::EngineError;

// Maximum Solana transaction size (MTU) in bytes.

pub const MAX_TX_MTU_SIZE: usize = 1232;

pub struct StackBuffer {
    pub memory: [u8; MAX_TX_MTU_SIZE],
    pub head: usize,
}

impl Default for StackBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl StackBuffer {
    pub fn new() -> Self {
        Self {
            memory: [0u8; MAX_TX_MTU_SIZE],
            head: 0,
        }
    }

    pub fn len(&self)->usize{
        self.head
    }

    pub fn is_empty(&self)->bool{
        self.head==0
    }
    pub fn as_slice(&self)->&[u8]{
     &self.memory[0..self.head]   
    }
}

/// Trait for ingesting a Base64‑encoded transaction payload into a stack buffer.
pub trait PayloadIngestion: Send + Sync {
    fn decode_transaction_in_place<'a>(
        &self,
        b64_payload: &str,
        buffer: &'a mut StackBuffer,
    ) -> Result<&'a [u8], EngineError>;
}

/// Implementation using the standard Base64 engine with zero‑allocation.

pub struct Base64PayloadIngestion;

impl PayloadIngestion for Base64PayloadIngestion {
    fn decode_transaction_in_place<'a>(
        &self,
        b64_payload: &str,
        buffer: &'a mut StackBuffer,
    ) -> Result<&'a [u8], EngineError> {
        let bytes_written = BASE64_STANDARD
            .decode_slice(b64_payload.as_bytes(), &mut buffer.memory)
            .map_err(|e| match e {
                base64::DecodeSliceError::OutputSliceTooSmall => EngineError::TransactionTooLarge,
                base64::DecodeSliceError::DecodeError(_err) => EngineError::PayloadFormatInvalid,
            })?;

        buffer.head = bytes_written;
        Ok(&buffer.memory[..bytes_written])
    }
}
    

// ---------- Transaction Validator ----------

/// Validates and deserializes a transaction from raw bytes.
/// Enforces strict bincode rules: no trailing bytes, size limit.
pub trait TransactionValidator {
    fn verify_and_deserialize(
        raw_tx_bytes: &[u8],
    ) -> Result<VersionedTransaction, EngineError>;
}

/// Default implementation using bincode with limit and trailing‑byte rejection.

pub struct BincodeTransactionValidator;

impl TransactionValidator for BincodeTransactionValidator {
    fn verify_and_deserialize(raw_tx_bytes: &[u8]) -> Result<VersionedTransaction, EngineError> {
        // Use bincode options: limit size, reject trailing bytes.
        let options = bincode::DefaultOptions::new()
            .with_limit(MAX_TX_MTU_SIZE as u64)
            .reject_trailing_bytes();

        options
            .deserialize(raw_tx_bytes)
            .map_err(|_e| EngineError::PayloadFormatInvalid) 
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use solana_sdk::signature::Keypair;
    use solana_sdk::transaction::VersionedTransaction;

    #[test]
    fn test_decode_ok() {
        let raw = b"hello world";
        let b64 = BASE64_STANDARD.encode(raw);
        let mut buffer = StackBuffer::new();
        let ingestor = Base64PayloadIngestion;
        let result = ingestor.decode_transaction_in_place(&b64, &mut buffer).unwrap();
        assert_eq!(result, raw);
        assert_eq!(buffer.len(), raw.len());
    }

    #[test]
    fn test_size_exceeded() {
        let large = vec![0u8; MAX_TX_MTU_SIZE + 1];
        let b64 = BASE64_STANDARD.encode(&large);
        let mut buffer = StackBuffer::new();
        let ingestor = Base64PayloadIngestion;
        let err = ingestor.decode_transaction_in_place(&b64, &mut buffer).unwrap_err();
        assert!(matches!(err, EngineError::TransactionTooLarge));
    }

    #[test]
    fn test_validate_ok() {
        // Create a dummy VersionedTransaction (minimal).
        let _keypair = Keypair::new();
        let tx = VersionedTransaction::default();
        let bytes = bincode::serialize(&tx).unwrap();
        let result = BincodeTransactionValidator::verify_and_deserialize(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_trailing_bytes() {
        let _keypair = Keypair::new();
        let tx = VersionedTransaction::default();
        let mut bytes = bincode::serialize(&tx).unwrap();
        bytes.push(42); // trailing byte
        let result = BincodeTransactionValidator::verify_and_deserialize(&bytes);
        assert!(matches!(result, Err(EngineError::PayloadFormatInvalid)));
    }
}
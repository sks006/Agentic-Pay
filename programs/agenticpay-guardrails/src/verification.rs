use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    ed25519_program::ID as ED25519_PROGRAM_ID,
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
};

use crate::constants::{CANONICAL_MESSAGE_LEN, DOMAIN_SEPARATOR};
use crate::error::GuardrailError;

/// Verify that the preceding instruction is a valid Ed25519 verification.
pub fn verify_ed25519_ix(
    instructions_sysvar: &AccountInfo,
    message: &[u8],
    expected_pubkey: &[u8; 32],
) -> Result<()> {
    let current_idx = load_current_index_checked(instructions_sysvar)
        .map_err(|_| error!(GuardrailError::MissingEd25519Instruction))?;
    let ed25519_idx = current_idx
        .checked_sub(1)
        .ok_or(error!(GuardrailError::MissingEd25519Instruction))?;
    let ed25519_ix = load_instruction_at_checked(ed25519_idx as usize, instructions_sysvar)
        .map_err(|_| error!(GuardrailError::MissingEd25519Instruction))?;

    require_keys_eq!(
        ed25519_ix.program_id,
        ED25519_PROGRAM_ID,
        GuardrailError::MissingEd25519Instruction
    );

    let data = ed25519_ix.data;
    require!(data.len() >= 16, GuardrailError::InvalidEd25519Instruction);
    require!(data[0] == 1, GuardrailError::InvalidEd25519Instruction);

    let pk_offset = u16::from_le_bytes([data[6], data[7]]) as usize;
    let msg_offset = u16::from_le_bytes([data[10], data[11]]) as usize;
    let msg_size = u16::from_le_bytes([data[12], data[13]]) as usize;

    require!(
        data.len() >= msg_offset + msg_size,
        GuardrailError::InvalidEd25519Instruction
    );

    let signed_message = &data[msg_offset..msg_offset + msg_size];
    require!(signed_message == message, GuardrailError::MessageMismatch);

    require!(
        data.len() >= pk_offset + 32,
        GuardrailError::InvalidEd25519Instruction
    );
    let pubkey_bytes = &data[pk_offset..pk_offset + 32];
    require!(pubkey_bytes == expected_pubkey, GuardrailError::AgentPubkeyMismatch);

    Ok(())
}

/// Verify the canonical message matches the expected layout.
pub fn verify_canonical_message(
    message: &[u8],
    nonce: u64,
    agent: &[u8; 32],
    provider: &[u8; 32],
    amount_lamports: u64,
    expires_at: i64,
) -> Result<()> {
    require!(message.len() == CANONICAL_MESSAGE_LEN, GuardrailError::MessageMismatch);
    require!(&message[0..17] == DOMAIN_SEPARATOR, GuardrailError::MessageMismatch);
    require!(&message[17..25] == &nonce.to_le_bytes(), GuardrailError::MessageMismatch);
    require!(&message[25..57] == agent, GuardrailError::MessageMismatch);
    require!(&message[57..89] == provider, GuardrailError::MessageMismatch);
    require!(&message[89..97] == &amount_lamports.to_le_bytes(), GuardrailError::MessageMismatch);
    require!(&message[97..105] == &expires_at.to_le_bytes(), GuardrailError::MessageMismatch);
    Ok(())
}
import nacl from "tweetnacl";
import { PublicKey } from "@solana/web3.js";

export const DOMAIN_SEPARATOR = Buffer.from("agenticpay_x402v1");
export const CANONICAL_MESSAGE_LEN = 105;
export const VOUCHER_TOTAL_LEN = 170; // 105 canonical + 64 signature + 1 retry_count

export interface VoucherPayload {
  nonce: bigint;
  agent: Buffer; // 32 bytes
  provider: Buffer; // 32 bytes
  amountLamports: bigint;
  expiresAt: bigint; // Unix timestamp in seconds
  signature: Buffer; // 64 bytes
  retryCount: number; // 1 byte
}

/**
 * Construct the canonical 105-byte message for signing / verification.
 * Must match agent-backend/src/voucher.rs byte-for-byte.
 */
export function createCanonicalMessage(
  nonce: bigint,
  agent: Buffer,
  provider: Buffer,
  amountLamports: bigint,
  expiresAt: bigint
): Buffer {
  if (agent.length !== 32) {
    throw new Error(`Agent pubkey must be 32 bytes, got ${agent.length}`);
  }
  if (provider.length !== 32) {
    throw new Error(`Provider pubkey must be 32 bytes, got ${provider.length}`);
  }

  const buf = Buffer.alloc(CANONICAL_MESSAGE_LEN);
  let offset = 0;

  DOMAIN_SEPARATOR.copy(buf, offset);
  offset += 17;

  buf.writeBigUInt64LE(nonce, offset);
  offset += 8;

  agent.copy(buf, offset);
  offset += 32;

  provider.copy(buf, offset);
  offset += 32;

  buf.writeBigUInt64LE(amountLamports, offset);
  offset += 8;

  buf.writeBigInt64LE(expiresAt, offset);
  offset += 8;

  return buf;
}

/**
 * Get canonical bytes directly from a VoucherPayload.
 */
export function getVoucherCanonicalBytes(voucher: VoucherPayload): Buffer {
  return createCanonicalMessage(
    voucher.nonce,
    voucher.agent,
    voucher.provider,
    voucher.amountLamports,
    voucher.expiresAt
  );
}

/**
 * Encode a VoucherPayload into a base64 string for the PAYMENT-SIGNATURE header.
 */
export function encodeVoucherHeader(voucher: VoucherPayload): string {
  const buf = Buffer.alloc(VOUCHER_TOTAL_LEN);
  const canonical = getVoucherCanonicalBytes(voucher);

  canonical.copy(buf, 0);
  voucher.signature.copy(buf, CANONICAL_MESSAGE_LEN);
  buf.writeUInt8(voucher.retryCount, CANONICAL_MESSAGE_LEN + 64);

  return buf.toString("base64");
}

/**
 * Decode a Base64-encoded voucher from the PAYMENT-SIGNATURE header.
 */
export function decodeVoucherHeader(encoded: string): VoucherPayload {
  const bytes = Buffer.from(encoded, "base64");

  if (bytes.length !== VOUCHER_TOTAL_LEN) {
    throw new Error(
      `Invalid voucher length: expected ${VOUCHER_TOTAL_LEN} bytes, received ${bytes.length}`
    );
  }

  // Check domain separator
  const prefix = bytes.subarray(0, 17);
  if (!prefix.equals(DOMAIN_SEPARATOR)) {
    throw new Error("Invalid domain separator in voucher");
  }

  const nonce = bytes.readBigUInt64LE(17);
  const agent = Buffer.from(bytes.subarray(25, 57));
  const provider = Buffer.from(bytes.subarray(57, 89));
  const amountLamports = bytes.readBigUInt64LE(89);
  const expiresAt = bytes.readBigInt64LE(97);
  const signature = Buffer.from(bytes.subarray(105, 169));
  const retryCount = bytes.readUInt8(169);

  return {
    nonce,
    agent,
    provider,
    amountLamports,
    expiresAt,
    signature,
    retryCount,
  };
}

/**
 * Verify Ed25519 signature of the voucher against canonical message and agent public key.
 */
export function verifyVoucher(voucher: VoucherPayload): boolean {
  const canonical = getVoucherCanonicalBytes(voucher);
  return nacl.sign.detached.verify(
    new Uint8Array(canonical),
    new Uint8Array(voucher.signature),
    new Uint8Array(voucher.agent)
  );
}

/**
 * Check if the voucher is expired.
 */
export function isExpired(
  voucher: VoucherPayload,
  nowSecs: number = Math.floor(Date.now() / 1000)
): boolean {
  return BigInt(nowSecs) > voucher.expiresAt;
}

/**
 * Helper to convert 32-byte Buffer to base58 Solana public key string.
 */
export function bufferToPubkeyString(buffer: Buffer): string {
  return new PublicKey(buffer).toBase58();
}

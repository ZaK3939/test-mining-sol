// Invite code hash utilities to match Rust implementation
// Implements SHA256 hashing for hash-based invite code system

import { PublicKey } from '@solana/web3.js';

/**
 * Fixed salt for invite code hashing (matches Rust implementation)
 * This is not meant to be secret - it simplifies PDA calculation
 */
const FIXED_SALT = new Uint8Array([
  0x46, 0x41, 0x43, 0x49, 0x4c, 0x49, 0x54, 0x59,  // "FACILITY"
  0x47, 0x41, 0x4d, 0x45, 0x32, 0x30, 0x32, 0x34,  // "GAME2024"
]);

/**
 * Generate a secure hash for invite codes
 * Formula: SHA256(plaintext_code + salt + inviter_pubkey)
 * This must match the Rust implementation exactly
 */
export async function generateInviteCodeHash(
  plaintextCode: Uint8Array, // 8 bytes
  salt: Uint8Array,          // 16 bytes
  inviterPubkey: PublicKey   // 32 bytes
): Promise<Uint8Array> {
  // Concatenate: code + salt + pubkey
  const data = new Uint8Array(8 + 16 + 32);
  data.set(plaintextCode, 0);
  data.set(salt, 8);
  data.set(inviterPubkey.toBytes(), 24);
  
  // Use Web Crypto API for SHA256
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  return new Uint8Array(hashBuffer);
}

/**
 * Get the fixed salt used for invite code hashing
 */
export function getFixedSalt(): Uint8Array {
  return FIXED_SALT;
}

/**
 * Validate invite code format (8 alphanumeric characters)
 */
export function validateInviteCodeFormat(inviteCode: string): boolean {
  if (inviteCode.length !== 8) return false;
  
  for (let i = 0; i < 8; i++) {
    const char = inviteCode.charCodeAt(i);
    const isAlphaNumeric = 
      (char >= 48 && char <= 57) ||  // 0-9
      (char >= 65 && char <= 90) ||  // A-Z
      (char >= 97 && char <= 122);   // a-z
    
    if (!isAlphaNumeric) return false;
  }
  
  return true;
}

/**
 * Convert string invite code to 8-byte array
 */
export function stringToInviteCodeBytes(inviteCode: string): Uint8Array {
  const padded = inviteCode.padEnd(8, '\0');
  const bytes = new Uint8Array(8);
  
  for (let i = 0; i < 8; i++) {
    bytes[i] = padded.charCodeAt(i);
  }
  
  return bytes;
}

/**
 * Convert 8-byte array to string invite code
 */
export function inviteCodeBytesToString(bytes: Uint8Array): string {
  let result = '';
  
  for (let i = 0; i < Math.min(8, bytes.length); i++) {
    if (bytes[i] === 0) break; // Stop at null terminator
    result += String.fromCharCode(bytes[i]);
  }
  
  return result;
}

/**
 * Verify that a plaintext code matches the stored hash
 */
export async function verifyInviteCodeHash(
  plaintextCode: Uint8Array,
  salt: Uint8Array,
  inviterPubkey: PublicKey,
  storedHash: Uint8Array
): Promise<boolean> {
  const computedHash = await generateInviteCodeHash(plaintextCode, salt, inviterPubkey);
  
  // Compare hashes byte by byte
  if (computedHash.length !== storedHash.length) return false;
  
  for (let i = 0; i < computedHash.length; i++) {
    if (computedHash[i] !== storedHash[i]) return false;
  }
  
  return true;
}

/**
 * Calculate the PDA for a hash-based invite code
 * Seeds: ["invite_hash", inviter_pubkey, hash]
 */
export async function calculateInviteCodePDA(
  inviteCode: string,
  inviterPubkey: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> {
  // Validate format first
  if (!validateInviteCodeFormat(inviteCode)) {
    throw new Error('Invalid invite code format. Must be 8 alphanumeric characters.');
  }
  
  // Convert to bytes and generate hash
  const codeBytes = stringToInviteCodeBytes(inviteCode);
  const salt = getFixedSalt();
  const hash = await generateInviteCodeHash(codeBytes, salt, inviterPubkey);
  
  // Calculate PDA with hash-based seeds
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from('invite_hash'),
      inviterPubkey.toBuffer(),
      Buffer.from(hash),
    ],
    programId
  );
}
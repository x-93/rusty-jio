//! Standard transaction script templates, classification, and execution engine.
//!
//! Implements evaluation for P2PK, P2PKH, P2SH, and Schnorr / ECDSA signature verifications.

pub mod multisig;

use crate::data_stack::{DataStack, StackError};
use crate::script_builder::{OpCode, ScriptBuilder};
use jio_addresses::{Address, Prefix, Version};
use jio_hashes::{Hash, Hasher};
use secp256k1::schnorr::Signature as SchnorrSignature;
use secp256k1::{Message, Secp256k1, XOnlyPublicKey};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum TxScriptError {
    #[error("stack error: {0}")]
    Stack(#[from] StackError),
    #[error("script execution failed: top of stack was false or stack was empty")]
    ScriptFailed,
    #[error("opcode verify failed")]
    VerifyFailed,
    #[error("invalid signature format")]
    InvalidSignature,
    #[error("invalid public key format")]
    InvalidPublicKey,
    #[error("signature verification failed")]
    BadSignature,
    #[error("invalid opcode {0:#x}")]
    InvalidOpcode(u8),
    #[error("unexpected end of script")]
    UnexpectedEndOfScript,
    #[error("script contains unexecuted OP_RETURN")]
    OpReturnExecuted,
    #[error("unsupported or non-standard script")]
    NonStandardScript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptClass {
    PubKey,
    PubKeyECDSA,
    ScriptHash,
    PubKeyHash,
    MultiSig,
    NonStandard,
}

pub fn pay_to_pub_key_script(pubkey: &[u8]) -> Vec<u8> {
    let mut builder = ScriptBuilder::new();
    builder.add_p2pk(pubkey);
    builder.drain()
}

pub fn pay_to_script_hash_script(script_hash: &[u8]) -> Vec<u8> {
    let mut builder = ScriptBuilder::new();
    builder.add_p2sh(script_hash);
    builder.drain()
}

pub fn pay_to_pubkey_hash_script(pubkey_hash: &[u8]) -> Vec<u8> {
    let mut builder = ScriptBuilder::new();
    builder
        .add_op(OpCode::OpDup)
        .add_op(OpCode::OpBlake2b)
        .add_data(pubkey_hash)
        .add_op(OpCode::OpEqualVerify)
        .add_op(OpCode::OpCheckSig);
    builder.drain()
}

/// Classify a script public key.
pub fn classify_script(script: &[u8]) -> ScriptClass {
    if script.len() == 34 && script[0] == 32 && script[33] == OpCode::OpCheckSig as u8 {
        ScriptClass::PubKey
    } else if script.len() == 35 && script[0] == 33 && script[34] == OpCode::OpCheckSig as u8 {
        ScriptClass::PubKeyECDSA
    } else if script.len() == 35
        && script[0] == OpCode::OpBlake2b as u8
        && script[1] == 32
        && script[34] == OpCode::OpEqual as u8
    {
        ScriptClass::ScriptHash
    } else if script.len() == 37
        && script[0] == OpCode::OpDup as u8
        && script[1] == OpCode::OpBlake2b as u8
        && script[2] == 32
        && script[35] == OpCode::OpEqualVerify as u8
        && script[36] == OpCode::OpCheckSig as u8
    {
        ScriptClass::PubKeyHash
    } else {
        ScriptClass::NonStandard
    }
}

/// Extract recipient address from a standard script public key.
pub fn extract_script_pub_key_address(script: &[u8], prefix: Prefix) -> Result<Address, TxScriptError> {
    match classify_script(script) {
        ScriptClass::PubKey => {
            let pubkey = &script[1..33];
            Address::new(prefix, Version::PubKey, pubkey).map_err(|_| TxScriptError::InvalidPublicKey)
        }
        ScriptClass::PubKeyECDSA => {
            let pubkey = &script[1..34];
            Address::new(prefix, Version::PubKeyECDSA, pubkey).map_err(|_| TxScriptError::InvalidPublicKey)
        }
        ScriptClass::ScriptHash => {
            let hash = &script[2..34];
            Address::new(prefix, Version::ScriptHash, hash).map_err(|_| TxScriptError::InvalidPublicKey)
        }
        _ => Err(TxScriptError::NonStandardScript),
    }
}

/// Verify a BIP-340 64-byte Schnorr signature against a 32-byte x-only public key.
pub fn verify_schnorr_signature(
    pubkey_bytes: &[u8],
    sig_bytes: &[u8],
    sighash: &[u8; 32],
) -> Result<bool, TxScriptError> {
    if pubkey_bytes.len() != 32 || sig_bytes.len() != 64 {
        return Ok(false);
    }
    let secp = Secp256k1::verification_only();
    let pubkey = XOnlyPublicKey::from_slice(pubkey_bytes).map_err(|_| TxScriptError::InvalidPublicKey)?;
    let sig = SchnorrSignature::from_slice(sig_bytes).map_err(|_| TxScriptError::InvalidSignature)?;
    let msg = Message::from_digest(*sighash);
    Ok(secp.verify_schnorr(&sig, &msg, &pubkey).is_ok())
}

/// Forth-like Script execution engine.
pub struct Engine<'a> {
    pub stack: DataStack,
    script_sig: &'a [u8],
    script_pub_key: &'a [u8],
    sighash: Hash,
}

impl<'a> Engine<'a> {
    pub fn new(script_sig: &'a [u8], script_pub_key: &'a [u8], sighash: Hash) -> Self {
        Self {
            stack: DataStack::new(),
            script_sig,
            script_pub_key,
            sighash,
        }
    }

    /// Execute both script_sig and script_pub_key and verify that the result is valid.
    pub fn execute(&mut self) -> Result<(), TxScriptError> {
        self.step_script(self.script_sig)?;

        let is_p2sh = classify_script(self.script_pub_key) == ScriptClass::ScriptHash;
        let p2sh_script = if is_p2sh {
            Some(self.stack.peek()?.to_vec())
        } else {
            None
        };

        self.step_script(self.script_pub_key)?;

        if is_p2sh {
            if !self.stack.pop_bool()? {
                return Err(TxScriptError::ScriptFailed);
            }
            if let Some(redeem) = p2sh_script {
                self.step_script(&redeem)?;
            }
        }

        if self.stack.is_empty() || !self.stack.pop_bool()? {
            Err(TxScriptError::ScriptFailed)
        } else {
            Ok(())
        }
    }

    fn step_script(&mut self, script: &[u8]) -> Result<(), TxScriptError> {
        let mut pc = 0;
        while pc < script.len() {
            let opcode = script[pc];
            pc += 1;

            // Push data opcodes
            if opcode > 0 && opcode <= 75 {
                let len = opcode as usize;
                if pc + len > script.len() {
                    return Err(TxScriptError::UnexpectedEndOfScript);
                }
                self.stack.push(script[pc..pc + len].to_vec());
                pc += len;
                continue;
            }

            match opcode {
                0x00 => self.stack.push_bool(false),
                0x4c => {
                    if pc >= script.len() {
                        return Err(TxScriptError::UnexpectedEndOfScript);
                    }
                    let len = script[pc] as usize;
                    pc += 1;
                    if pc + len > script.len() {
                        return Err(TxScriptError::UnexpectedEndOfScript);
                    }
                    self.stack.push(script[pc..pc + len].to_vec());
                    pc += len;
                }
                0x4d => {
                    if pc + 2 > script.len() {
                        return Err(TxScriptError::UnexpectedEndOfScript);
                    }
                    let len = u16::from_le_bytes(script[pc..pc + 2].try_into().unwrap()) as usize;
                    pc += 2;
                    if pc + len > script.len() {
                        return Err(TxScriptError::UnexpectedEndOfScript);
                    }
                    self.stack.push(script[pc..pc + len].to_vec());
                    pc += len;
                }
                0x4f => self.stack.push_i64(-1),
                0x51..=0x60 => {
                    let val = (opcode - 0x51 + 1) as i64;
                    self.stack.push_i64(val);
                }
                0x61 => {} // OP_NOP
                0x69 => {
                    // OP_VERIFY
                    if !self.stack.pop_bool()? {
                        return Err(TxScriptError::VerifyFailed);
                    }
                }
                0x6a => return Err(TxScriptError::OpReturnExecuted),
                0x6d => {
                    // OP_2DROP
                    self.stack.drop()?;
                    self.stack.drop()?;
                }
                0x6e => {
                    // OP_2DUP
                    let len = self.stack.len();
                    if len < 2 {
                        return Err(StackError::StackUnderflow.into());
                    }
                    self.stack.pick(1)?;
                    self.stack.pick(1)?;
                }
                0x75 => self.stack.drop()?,
                0x76 => self.stack.dup()?,
                0x77 => self.stack.nip()?,
                0x78 => self.stack.over()?,
                0x7c => self.stack.swap()?,
                0x7b => self.stack.rot()?,
                0x87 => {
                    // OP_EQUAL
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push_bool(a == b);
                }
                0x88 => {
                    // OP_EQUALVERIFY
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    if a != b {
                        return Err(TxScriptError::VerifyFailed);
                    }
                }
                0x8b => {
                    // OP_1ADD
                    let v = self.stack.pop_i64()?;
                    self.stack.push_i64(v + 1);
                }
                0x8c => {
                    // OP_1SUB
                    let v = self.stack.pop_i64()?;
                    self.stack.push_i64(v - 1);
                }
                0x8f => {
                    // OP_NEGATE
                    let v = self.stack.pop_i64()?;
                    self.stack.push_i64(-v);
                }
                0x91 => {
                    // OP_NOT
                    let v = self.stack.pop_bool()?;
                    self.stack.push_bool(!v);
                }
                0x93 => {
                    // OP_ADD
                    let b = self.stack.pop_i64()?;
                    let a = self.stack.pop_i64()?;
                    self.stack.push_i64(a + b);
                }
                0x94 => {
                    // OP_SUB
                    let b = self.stack.pop_i64()?;
                    let a = self.stack.pop_i64()?;
                    self.stack.push_i64(a - b);
                }
                0xaa => {
                    // OP_BLAKE2B
                    let data = self.stack.pop()?;
                    let hash = jio_hashes::TransactionHash::hash(&data);
                    self.stack.push(hash.as_bytes().to_vec());
                }
                0xac => {
                    // OP_CHECKSIG
                    let pubkey = self.stack.pop()?;
                    let sig = self.stack.pop()?;
                    let ok = verify_schnorr_signature(&pubkey, &sig, &self.sighash.as_bytes())?;
                    self.stack.push_bool(ok);
                }
                0xad => {
                    // OP_CHECKSIGVERIFY
                    let pubkey = self.stack.pop()?;
                    let sig = self.stack.pop()?;
                    let ok = verify_schnorr_signature(&pubkey, &sig, &self.sighash.as_bytes())?;
                    if !ok {
                        return Err(TxScriptError::BadSignature);
                    }
                }
                _ => return Err(TxScriptError::InvalidOpcode(opcode)),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::Keypair;

    #[test]
    fn test_p2pk_evaluation() {
        let secp = Secp256k1::new();
        let keypair = Keypair::new(&secp, &mut rand::thread_rng());
        let (x_only, _parity) = keypair.x_only_public_key();
        let pubkey_bytes = x_only.serialize();

        let sighash = Hash::from_u64_word(0x1337);
        let msg = Message::from_digest(sighash.as_bytes());
        let sig = secp.sign_schnorr(&msg, &keypair);
        let sig_bytes = sig.as_ref().to_vec();

        let script_pub_key = pay_to_pub_key_script(&pubkey_bytes);
        let script_sig = ScriptBuilder::new().add_data(&sig_bytes).drain();

        let mut engine = Engine::new(&script_sig, &script_pub_key, sighash);
        assert!(engine.execute().is_ok());

        // Invalid signature must fail
        let mut bad_sig = sig_bytes.clone();
        bad_sig[0] ^= 1;
        let bad_script_sig = ScriptBuilder::new().add_data(&bad_sig).drain();
        let mut bad_engine = Engine::new(&bad_script_sig, &script_pub_key, sighash);
        assert!(bad_engine.execute().is_err());
    }

    #[test]
    fn test_p2sh_evaluation() {
        let inner_script = ScriptBuilder::new()
            .add_i64(5)
            .add_i64(5)
            .add_op(OpCode::OpAdd)
            .add_i64(10)
            .add_op(OpCode::OpEqual)
            .drain();

        let script_hash = jio_hashes::TransactionHash::hash(&inner_script);
        let script_pub_key = pay_to_script_hash_script(&script_hash.as_bytes());
        let script_sig = ScriptBuilder::new().add_data(&inner_script).drain();

        let mut engine = Engine::new(&script_sig, &script_pub_key, Hash::default());
        assert!(engine.execute().is_ok());
    }
}

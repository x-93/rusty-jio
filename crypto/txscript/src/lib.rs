// lib.rs
//
// Bitcoin / Kaspa compatible transaction script (TxScript) execution engine.
// Executes unlocking and locking scripts to authenticate transaction inputs.

pub mod data_stack;
pub mod opcodes;
pub mod script_builder;
pub mod standard;

pub use data_stack::{DataStack, StackError};
pub use opcodes::codes as op;
pub use script_builder::ScriptBuilder;
pub use standard::*;

use jio_hashes::{Hash, HasherBase};
use secp256k1::schnorr::Signature as SchnorrSignature;
use secp256k1::{
    Message, PublicKey, Secp256k1, XOnlyPublicKey, ecdsa::Signature as EcdsaSignature,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ScriptError {
    #[error("Stack error: {0}")]
    Stack(#[from] StackError),
    #[error("Script failed verification (top element is false or empty)")]
    FailedVerification,
    #[error("Script verification aborted by OP_RETURN")]
    OpReturn,
    #[error("Unimplemented opcode: 0x{0:02x}")]
    UnimplementedOpcode(u8),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("Unsupported script version: {0}")]
    UnsupportedVersion(u16),
}

/// Abstract verification context providing transaction signing hashes for signature verification.
pub trait TxSignatureVerifier {
    /// Return the 32-byte signing hash for Schnorr signature verification
    fn compute_schnorr_sighash(&self) -> Hash;
    /// Return the 32-byte signing hash for ECDSA signature verification
    fn compute_ecdsa_sighash(&self) -> Hash;
}

/// Standalone test verifier using dummy sighashes
#[derive(Clone, Copy, Debug)]
pub struct MockSignatureVerifier {
    pub sighash: Hash,
}

impl TxSignatureVerifier for MockSignatureVerifier {
    fn compute_schnorr_sighash(&self) -> Hash {
        self.sighash
    }
    fn compute_ecdsa_sighash(&self) -> Hash {
        self.sighash
    }
}

/// Stack-based TxScript Execution Engine
#[derive(Clone, Debug, Default)]
pub struct TxScriptEngine {
    pub stack: DataStack,
}

impl TxScriptEngine {
    pub fn new() -> Self {
        Self {
            stack: DataStack::new(),
        }
    }

    /// Executes an unlocking signature script followed by a locking script public key.
    pub fn execute<V: TxSignatureVerifier>(
        &mut self,
        sig_script: &[u8],
        spk: &ScriptPublicKey,
        verifier: &V,
    ) -> Result<(), ScriptError> {
        if spk.version != 0 {
            return Err(ScriptError::UnsupportedVersion(spk.version));
        }

        // Phase 1: Execute SignatureScript (unlocking script)
        self.execute_script_bytes(sig_script, verifier)?;

        // Phase 2: Execute ScriptPublicKey (locking script)
        self.execute_script_bytes(spk.script.as_slice(), verifier)?;

        // Phase 3: Evaluate stack outcome
        let top = self.stack.pop()?;
        if is_true(&top) {
            Ok(())
        } else {
            Err(ScriptError::FailedVerification)
        }
    }

    /// Executes raw script bytecode.
    fn execute_script_bytes<V: TxSignatureVerifier>(
        &mut self,
        script: &[u8],
        verifier: &V,
    ) -> Result<(), ScriptError> {
        let mut pc = 0;
        let secp = Secp256k1::new();

        while pc < script.len() {
            let opcode = script[pc];
            pc += 1;

            match opcode {
                // Direct data pushes (1..=75 bytes)
                1..=75 => {
                    let len = opcode as usize;
                    if pc + len > script.len() {
                        return Err(ScriptError::Stack(StackError::Underflow));
                    }
                    self.stack.push(script[pc..pc + len].to_vec())?;
                    pc += len;
                }
                op::OP_0 => {
                    self.stack.push(Vec::new())?;
                }
                op::OP_1 => {
                    self.stack.push(vec![1])?;
                }
                op::OP_PUSHDATA1 => {
                    if pc >= script.len() {
                        return Err(ScriptError::Stack(StackError::Underflow));
                    }
                    let len = script[pc] as usize;
                    pc += 1;
                    if pc + len > script.len() {
                        return Err(ScriptError::Stack(StackError::Underflow));
                    }
                    self.stack.push(script[pc..pc + len].to_vec())?;
                    pc += len;
                }
                op::OP_PUSHDATA2 => {
                    if pc + 2 > script.len() {
                        return Err(ScriptError::Stack(StackError::Underflow));
                    }
                    let len = u16::from_le_bytes(script[pc..pc + 2].try_into().unwrap()) as usize;
                    pc += 2;
                    if pc + len > script.len() {
                        return Err(ScriptError::Stack(StackError::Underflow));
                    }
                    self.stack.push(script[pc..pc + len].to_vec())?;
                    pc += len;
                }
                op::OP_NOP => {}
                op::OP_RETURN => {
                    return Err(ScriptError::OpReturn);
                }
                op::OP_DUP => {
                    self.stack.dup()?;
                }
                op::OP_DROP => {
                    self.stack.drop()?;
                }
                op::OP_SWAP => {
                    self.stack.swap()?;
                }
                op::OP_EQUAL => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(if a == b { vec![1] } else { vec![] })?;
                }
                op::OP_EQUALVERIFY => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    if a != b {
                        return Err(ScriptError::FailedVerification);
                    }
                }
                op::OP_BLAKE2B => {
                    let item = self.stack.pop()?;
                    let mut h = jio_hashes::TransactionHash::new();
                    h.update(&item);
                    let res = h.finalize();
                    self.stack.push(res.as_bytes().to_vec())?;
                }
                op::OP_SHA256 => {
                    let item = self.stack.pop()?;
                    let mut h = jio_hashes::TransactionSigningHashECDSA::new();
                    h.update(&item);
                    let res = h.finalize();
                    self.stack.push(res.as_bytes().to_vec())?;
                }
                op::OP_CHECKSIG => {
                    let pk_bytes = self.stack.pop()?;
                    let sig_bytes = self.stack.pop()?;

                    if pk_bytes.len() != 32 {
                        return Err(ScriptError::InvalidPublicKey(
                            "Expected 32-byte Schnorr pubkey".into(),
                        ));
                    }
                    let xonly_pk = XOnlyPublicKey::from_slice(&pk_bytes)
                        .map_err(|e| ScriptError::InvalidPublicKey(e.to_string()))?;

                    let sig = SchnorrSignature::from_slice(&sig_bytes)
                        .map_err(|e| ScriptError::InvalidSignature(e.to_string()))?;

                    let sighash = verifier.compute_schnorr_sighash();
                    let msg = Message::from_digest_slice(&sighash.as_bytes()).unwrap();

                    let valid = secp.verify_schnorr(&sig, &msg, &xonly_pk).is_ok();
                    self.stack.push(if valid { vec![1] } else { vec![] })?;
                }
                op::OP_CHECKSIG_ECDSA => {
                    let pk_bytes = self.stack.pop()?;
                    let sig_bytes = self.stack.pop()?;

                    let pk = PublicKey::from_slice(&pk_bytes)
                        .map_err(|e| ScriptError::InvalidPublicKey(e.to_string()))?;

                    let sig = EcdsaSignature::from_der(&sig_bytes)
                        .or_else(|_| EcdsaSignature::from_compact(&sig_bytes))
                        .map_err(|e| ScriptError::InvalidSignature(e.to_string()))?;

                    let sighash = verifier.compute_ecdsa_sighash();
                    let msg = Message::from_digest_slice(&sighash.as_bytes()).unwrap();

                    let valid = secp.verify_ecdsa(&msg, &sig, &pk).is_ok();
                    self.stack.push(if valid { vec![1] } else { vec![] })?;
                }
                _ => return Err(ScriptError::UnimplementedOpcode(opcode)),
            }
        }

        Ok(())
    }
}

/// Evaluates whether a stack byte vector is considered "truthy" in Script.
#[inline(always)]
pub fn is_true(v: &[u8]) -> bool {
    for &b in v {
        if b != 0 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::Keypair;

    #[test]
    fn test_p2pk_schnorr_execution() {
        let secp = Secp256k1::new();
        let keypair = Keypair::new(&secp, &mut secp256k1::rand::thread_rng());
        let (xonly_pk, _) = keypair.x_only_public_key();

        let sighash = Hash::from_le_u64([10, 20, 30, 40]);
        let msg = Message::from_digest_slice(&sighash.as_bytes()).unwrap();
        let sig = secp.sign_schnorr(&msg, &keypair);

        // Locking script: <pubkey> OP_CHECKSIG
        let spk = pay_to_pubkey_script(&xonly_pk.serialize());

        // Unlocking script: <signature>
        let sig_script = ScriptBuilder::new().add_data(&sig[..]).drain();

        let verifier = MockSignatureVerifier { sighash };
        let mut engine = TxScriptEngine::new();

        assert!(engine.execute(&sig_script, &spk, &verifier).is_ok());
    }

    #[test]
    fn test_p2pk_schnorr_invalid_sig() {
        let secp = Secp256k1::new();
        let keypair = Keypair::new(&secp, &mut secp256k1::rand::thread_rng());
        let (xonly_pk, _) = keypair.x_only_public_key();

        let sighash = Hash::from_le_u64([10, 20, 30, 40]);
        let wrong_sighash = Hash::from_le_u64([99, 99, 99, 99]);
        let msg = Message::from_digest_slice(&sighash.as_bytes()).unwrap();
        let sig = secp.sign_schnorr(&msg, &keypair);

        let spk = pay_to_pubkey_script(&xonly_pk.serialize());
        let sig_script = ScriptBuilder::new().add_data(&sig[..]).drain();

        // Verification fails with wrong sighash
        let verifier = MockSignatureVerifier {
            sighash: wrong_sighash,
        };
        let mut engine = TxScriptEngine::new();

        assert_eq!(
            engine.execute(&sig_script, &spk, &verifier),
            Err(ScriptError::FailedVerification)
        );
    }

    #[test]
    fn test_p2sh_script_execution() {
        let secret = b"super_secret_preimage";
        let mut h = jio_hashes::TransactionHash::new();
        h.update(secret);
        let secret_hash = h.finalize();

        // Locking script: OP_BLAKE2B <secret_hash> OP_EQUAL
        let spk = pay_to_script_hash_script(&secret_hash.as_bytes());

        // Unlocking script: <secret>
        let sig_script = ScriptBuilder::new().add_data(secret).drain();

        let verifier = MockSignatureVerifier {
            sighash: Hash::from_bytes([0; 32]),
        };
        let mut engine = TxScriptEngine::new();

        assert!(engine.execute(&sig_script, &spk, &verifier).is_ok());
    }
}

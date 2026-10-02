//! # Jio TxScript
//!
//! Bitcoin-compatible script execution engine, opcodes, and standard script matchers.

pub mod caches;
pub mod data_stack;
pub mod error;
pub mod opcodes;
pub mod result;
pub mod script_builder;
pub mod script_class;
pub mod standard;

pub use data_stack::DataStack;
pub use error::ScriptError;
pub use result::ScriptResult;
pub use script_builder::ScriptBuilder;
pub use script_class::ScriptClass;
pub use standard::{extract_script_pub_key_address, is_standard_output, pay_to_address_script};

use opcodes::macros::*;
use secp256k1::schnorr::Signature as SchnorrSignature;
use secp256k1::{Message, XOnlyPublicKey};

/// Evaluates a signature script and public key script against a transaction sighash.
pub fn evaluate_script(
    signature_script: &[u8],
    public_key_script: &[u8],
    sighash: &[u8; 32],
) -> Result<(), ScriptError> {
    let mut stack = DataStack::new();

    // Step 1: Execute signature script (pushing data to stack)
    execute_push_only(signature_script, &mut stack)?;

    // Step 2: Execute public key script with the prepared stack
    execute_script(public_key_script, &mut stack, sighash)?;

    // Step 3: Verify stack top is truthy
    if stack.is_empty() {
        return Err(ScriptError::ScriptFailed);
    }
    let top = stack.pop()?;
    if !DataStack::is_true(&top) {
        return Err(ScriptError::ScriptFailed);
    }

    Ok(())
}

fn execute_push_only(script: &[u8], stack: &mut DataStack) -> Result<(), ScriptError> {
    let mut pc = 0;
    while pc < script.len() {
        let op = script[pc];
        pc += 1;

        if (OP_DATA_1..=OP_DATA_75).contains(&op) {
            let len = op as usize;
            if pc + len > script.len() {
                return Err(ScriptError::ScriptFailed);
            }
            stack.push(script[pc..pc + len].to_vec())?;
            pc += len;
        } else if op == OP_PUSHDATA1 {
            if pc >= script.len() {
                return Err(ScriptError::ScriptFailed);
            }
            let len = script[pc] as usize;
            pc += 1;
            if pc + len > script.len() {
                return Err(ScriptError::ScriptFailed);
            }
            stack.push(script[pc..pc + len].to_vec())?;
            pc += len;
        } else {
            return Err(ScriptError::DisabledOpcode(op));
        }
    }
    Ok(())
}

fn execute_script(script: &[u8], stack: &mut DataStack, sighash: &[u8; 32]) -> Result<(), ScriptError> {
    let mut pc = 0;
    while pc < script.len() {
        let op = script[pc];
        pc += 1;

        match op {
            OP_0 => stack.push(Vec::new())?,
            OP_1..=OP_16 => stack.push(vec![op - OP_1 + 1])?,
            OP_DUP => stack.dup()?,
            OP_DROP => stack.drop()?,
            OP_EQUAL | OP_EQUALVERIFY => {
                let a = stack.pop()?;
                let b = stack.pop()?;
                let equal = a == b;
                if op == OP_EQUALVERIFY && !equal {
                    return Err(ScriptError::ScriptFailed);
                }
                if op == OP_EQUAL {
                    stack.push(if equal { vec![1] } else { Vec::new() })?;
                }
            }
            OP_CHECKSIG | OP_CHECKSIGVERIFY => {
                let pubkey_bytes = stack.pop()?;
                let sig_bytes = stack.pop()?;

                let verified = verify_schnorr_signature(&pubkey_bytes, &sig_bytes, sighash);
                if op == OP_CHECKSIGVERIFY && !verified {
                    return Err(ScriptError::ScriptFailed);
                }
                if op == OP_CHECKSIG {
                    stack.push(if verified { vec![1] } else { Vec::new() })?;
                }
            }
            len if (OP_DATA_1..=OP_DATA_75).contains(&len) => {
                let l = len as usize;
                if pc + l > script.len() {
                    return Err(ScriptError::ScriptFailed);
                }
                stack.push(script[pc..pc + l].to_vec())?;
                pc += l;
            }
            _ => return Err(ScriptError::InvalidOpcode(op)),
        }
    }
    Ok(())
}

fn verify_schnorr_signature(pubkey_bytes: &[u8], sig_bytes: &[u8], sighash: &[u8; 32]) -> bool {
    if pubkey_bytes.len() != 32 || sig_bytes.len() != 64 {
        return false;
    }
    let secp = secp256k1::Secp256k1::verification_only();
    let Ok(xonly_pk) = XOnlyPublicKey::from_slice(pubkey_bytes) else {
        return false;
    };
    let Ok(sig) = SchnorrSignature::from_slice(sig_bytes) else {
        return false;
    };
    let msg = Message::from_digest(*sighash);
    secp.verify_schnorr(&sig, &msg, &xonly_pk).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_builder_and_class() {
        let pk = [0x77u8; 32];
        let script = ScriptBuilder::new().add_p2pk(&pk).drain();
        assert_eq!(ScriptClass::from_script(&script), ScriptClass::PubKey);
    }
}

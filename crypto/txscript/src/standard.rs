pub mod multisig;

pub use multisig::*;

use crate::opcodes::macros::Opcode;
use crate::script_builder::ScriptBuilder;
use crate::script_class::ScriptPublicKey;

pub fn pay_to_pubkey_script(pubkey: &[u8; 32]) -> ScriptPublicKey {
    let mut builder = ScriptBuilder::new();
    builder.add_data(pubkey);
    builder.add_op(Opcode::OpCheckSig);
    ScriptPublicKey::new(0, builder.into_vec())
}

pub fn pay_to_pubkey_ecdsa_script(pubkey: &[u8; 33]) -> ScriptPublicKey {
    let mut builder = ScriptBuilder::new();
    builder.add_data(pubkey);
    builder.add_op(Opcode::OpCheckSigECDSA);
    ScriptPublicKey::new(0, builder.into_vec())
}

pub fn pay_to_script_hash_script(script_hash: &[u8; 32]) -> ScriptPublicKey {
    let mut builder = ScriptBuilder::new();
    builder.add_op(Opcode::OpHash256);
    builder.add_data(script_hash);
    builder.add_op(Opcode::OpEqual);
    ScriptPublicKey::new(0, builder.into_vec())
}

pub fn pay_to_op_return_script(data: &[u8]) -> ScriptPublicKey {
    let mut builder = ScriptBuilder::new();
    builder.add_op(Opcode::OpReturn);
    builder.add_data(data);
    ScriptPublicKey::new(0, builder.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script_class::{classify_script, ScriptClass};

    #[test]
    fn test_pay_to_pubkey_classification() {
        let pk = [7u8; 32];
        let spk = pay_to_pubkey_script(&pk);
        assert_eq!(classify_script(&spk.script), ScriptClass::PubKey);
    }

    #[test]
    fn test_pay_to_script_hash_classification() {
        let sh = [9u8; 32];
        let spk = pay_to_script_hash_script(&sh);
        assert_eq!(classify_script(&spk.script), ScriptClass::ScriptHash);
    }
}

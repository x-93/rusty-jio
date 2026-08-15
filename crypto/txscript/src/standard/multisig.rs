use crate::opcodes::macros::Opcode;
use crate::script_builder::ScriptBuilder;
use crate::ScriptPublicKey;

pub fn pay_to_multisig_script(required: usize, pubkeys: &[[u8; 32]]) -> ScriptPublicKey {
    let mut builder = ScriptBuilder::new();
    builder.add_i64(required as i64);
    for pk in pubkeys {
        builder.add_data(pk);
    }
    builder.add_i64(pubkeys.len() as i64);
    builder.add_op(Opcode::OpCheckMultiSig);
    ScriptPublicKey::new(0, builder.into_vec())
}

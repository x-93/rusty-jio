//! Multi-signature standard script utilities.

use super::super::script_builder::ScriptBuilder;
use super::super::opcodes::macros::*;

/// Creates an m-of-n multisig script.
pub fn create_multisig_script(required: u8, pubkeys: &[&[u8]]) -> Vec<u8> {
    assert!(required > 0 && required <= pubkeys.len() as u8);
    let mut builder = ScriptBuilder::new();
    builder = builder.add_op(OP_1 + (required - 1));
    for pk in pubkeys {
        builder = builder.add_data(pk);
    }
    builder = builder.add_op(OP_1 + (pubkeys.len() as u8 - 1));
    builder = builder.add_op(OP_CHECKMULTISIG);
    builder.drain()
}

//! Jio transaction scripting engine, Forth-like stack machine, and standard scripts.

pub mod data_stack;
pub mod script_builder;
pub mod standard;

pub use data_stack::{DataStack, StackError};
pub use script_builder::{OpCode, ScriptBuilder};
pub use standard::multisig::{eval_multisig, pay_to_multisig_script};
pub use standard::{
    classify_script, extract_script_pub_key_address, pay_to_pub_key_script,
    pay_to_pubkey_hash_script, pay_to_script_hash_script, verify_schnorr_signature,
    Engine, ScriptClass, TxScriptError,
};

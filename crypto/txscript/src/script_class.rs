use crate::opcodes::macros::Opcode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScriptClass {
    PubKey,
    PubKeyECDSA,
    ScriptHash,
    MultiSig,
    OpReturn,
    NonStandard,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptPublicKey {
    pub version: u16,
    pub script: Vec<u8>,
}

impl ScriptPublicKey {
    pub fn new(version: u16, script: Vec<u8>) -> Self {
        Self { version, script }
    }

    pub fn from_vec(version: u16, script: Vec<u8>) -> Self {
        Self { version, script }
    }

    pub fn script(&self) -> &[u8] {
        &self.script
    }

    pub fn version(&self) -> u16 {
        self.version
    }
}

pub fn classify_script(script: &[u8]) -> ScriptClass {
    let len = script.len();
    if len == 0 {
        return ScriptClass::NonStandard;
    }

    if script[0] == u8::from(Opcode::OpReturn) {
        return ScriptClass::OpReturn;
    }

    // P2PK (Schnorr 32-byte pubkey + OpCheckSig): 0x20 <32 bytes> 0xac
    if len == 34 && script[0] == 0x20 && script[33] == u8::from(Opcode::OpCheckSig) {
        return ScriptClass::PubKey;
    }

    // P2PK ECDSA (33-byte compressed pubkey + OpCheckSigECDSA): 0x21 <33 bytes> 0xab
    if len == 35 && script[0] == 0x21 && script[34] == u8::from(Opcode::OpCheckSigECDSA) {
        return ScriptClass::PubKeyECDSA;
    }

    // P2SH (OpBlake2b / OpHash160 / OpEqual): OpBlake2b <32 bytes> OpEqual: 0xaa 0x20 <32 bytes> 0x87
    if len == 35
        && script[0] == u8::from(Opcode::OpHash256)
        && script[1] == 0x20
        && script[34] == u8::from(Opcode::OpEqual)
    {
        return ScriptClass::ScriptHash;
    }

    ScriptClass::NonStandard
}

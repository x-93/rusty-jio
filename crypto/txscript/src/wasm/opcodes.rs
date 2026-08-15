use crate::opcodes::macros::Opcode;

pub fn opcode_from_name(name: &str) -> Option<Opcode> {
    match name {
        "OpReturn" => Some(Opcode::OpReturn),
        "OpCheckSig" => Some(Opcode::OpCheckSig),
        "OpCheckSigECDSA" => Some(Opcode::OpCheckSigECDSA),
        "OpHash256" => Some(Opcode::OpHash256),
        "OpEqual" => Some(Opcode::OpEqual),
        "OpDup" => Some(Opcode::OpDup),
        _ => None,
    }
}

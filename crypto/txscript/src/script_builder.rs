//! Forth-like script bytecode builder and opcode definitions.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    OpFalse = 0x00,
    OpPushData1 = 0x4c,
    OpPushData2 = 0x4d,
    OpPushData4 = 0x4e,
    Op1Negate = 0x4f,
    Op1 = 0x51,
    Op2 = 0x52,
    Op3 = 0x53,
    Op4 = 0x54,
    Op5 = 0x55,
    Op6 = 0x56,
    Op7 = 0x57,
    Op8 = 0x58,
    Op9 = 0x59,
    Op10 = 0x5a,
    Op11 = 0x5b,
    Op12 = 0x5c,
    Op13 = 0x5d,
    Op14 = 0x5e,
    Op15 = 0x5f,
    Op16 = 0x60,
    OpNop = 0x61,
    OpVerify = 0x69,
    OpReturn = 0x6a,
    Op2Drop = 0x6d,
    Op2Dup = 0x6e,
    OpDrop = 0x75,
    OpDup = 0x76,
    OpNip = 0x77,
    OpOver = 0x78,
    OpPick = 0x79,
    OpRoll = 0x7a,
    OpRot = 0x7b,
    OpSwap = 0x7c,
    OpTuck = 0x7d,
    OpEqual = 0x87,
    OpEqualVerify = 0x88,
    Op1Add = 0x8b,
    Op1Sub = 0x8c,
    OpNegate = 0x8f,
    OpAbs = 0x90,
    OpNot = 0x91,
    Op0NotEqual = 0x92,
    OpAdd = 0x93,
    OpSub = 0x94,
    OpMul = 0x95,
    OpDiv = 0x96,
    OpMod = 0x97,
    OpBoolAnd = 0x9a,
    OpBoolOr = 0x9b,
    OpNumEqual = 0x9c,
    OpNumEqualVerify = 0x9d,
    OpNumNotEqual = 0x9e,
    OpLessThan = 0x9f,
    OpGreaterThan = 0xa0,
    OpLessThanOrEqual = 0xa1,
    OpGreaterThanOrEqual = 0xa2,
    OpSHA256 = 0xa8,
    OpBlake2b = 0xaa,
    OpCheckSig = 0xac,
    OpCheckSigVerify = 0xad,
    OpCheckMultiSig = 0xae,
    OpCheckMultiSigVerify = 0xaf,
}

impl OpCode {
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::OpFalse),
            0x4c => Some(Self::OpPushData1),
            0x4d => Some(Self::OpPushData2),
            0x4e => Some(Self::OpPushData4),
            0x4f => Some(Self::Op1Negate),
            0x51..=0x60 => {
                let offset = byte - 0x51;
                Some(match offset {
                    0 => Self::Op1,
                    1 => Self::Op2,
                    2 => Self::Op3,
                    3 => Self::Op4,
                    4 => Self::Op5,
                    5 => Self::Op6,
                    6 => Self::Op7,
                    7 => Self::Op8,
                    8 => Self::Op9,
                    9 => Self::Op10,
                    10 => Self::Op11,
                    11 => Self::Op12,
                    12 => Self::Op13,
                    13 => Self::Op14,
                    14 => Self::Op15,
                    _ => Self::Op16,
                })
            }
            0x61 => Some(Self::OpNop),
            0x69 => Some(Self::OpVerify),
            0x6a => Some(Self::OpReturn),
            0x6d => Some(Self::Op2Drop),
            0x6e => Some(Self::Op2Dup),
            0x75 => Some(Self::OpDrop),
            0x76 => Some(Self::OpDup),
            0x77 => Some(Self::OpNip),
            0x78 => Some(Self::OpOver),
            0x79 => Some(Self::OpPick),
            0x7a => Some(Self::OpRoll),
            0x7b => Some(Self::OpRot),
            0x7c => Some(Self::OpSwap),
            0x7d => Some(Self::OpTuck),
            0x87 => Some(Self::OpEqual),
            0x88 => Some(Self::OpEqualVerify),
            0x8b => Some(Self::Op1Add),
            0x8c => Some(Self::Op1Sub),
            0x8f => Some(Self::OpNegate),
            0x90 => Some(Self::OpAbs),
            0x91 => Some(Self::OpNot),
            0x92 => Some(Self::Op0NotEqual),
            0x93 => Some(Self::OpAdd),
            0x94 => Some(Self::OpSub),
            0x95 => Some(Self::OpMul),
            0x96 => Some(Self::OpDiv),
            0x97 => Some(Self::OpMod),
            0x9a => Some(Self::OpBoolAnd),
            0x9b => Some(Self::OpBoolOr),
            0x9c => Some(Self::OpNumEqual),
            0x9d => Some(Self::OpNumEqualVerify),
            0x9e => Some(Self::OpNumNotEqual),
            0x9f => Some(Self::OpLessThan),
            0xa0 => Some(Self::OpGreaterThan),
            0xa1 => Some(Self::OpLessThanOrEqual),
            0xa2 => Some(Self::OpGreaterThanOrEqual),
            0xa8 => Some(Self::OpSHA256),
            0xaa => Some(Self::OpBlake2b),
            0xac => Some(Self::OpCheckSig),
            0xad => Some(Self::OpCheckSigVerify),
            0xae => Some(Self::OpCheckMultiSig),
            0xaf => Some(Self::OpCheckMultiSigVerify),
            _ => None,
        }
    }
}

/// Fluent builder for script bytecode.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptBuilder {
    script: Vec<u8>,
}

impl ScriptBuilder {
    pub fn new() -> Self {
        Self { script: Vec::new() }
    }

    pub fn add_op(&mut self, op: OpCode) -> &mut Self {
        self.script.push(op as u8);
        self
    }

    pub fn add_data(&mut self, data: &[u8]) -> &mut Self {
        let len = data.len();
        if len == 0 {
            self.script.push(OpCode::OpFalse as u8);
        } else if len <= 75 {
            self.script.push(len as u8);
            self.script.extend_from_slice(data);
        } else if len <= 0xff {
            self.script.push(OpCode::OpPushData1 as u8);
            self.script.push(len as u8);
            self.script.extend_from_slice(data);
        } else if len <= 0xffff {
            self.script.push(OpCode::OpPushData2 as u8);
            self.script.extend_from_slice(&(len as u16).to_le_bytes());
            self.script.extend_from_slice(data);
        } else {
            self.script.push(OpCode::OpPushData4 as u8);
            self.script.extend_from_slice(&(len as u32).to_le_bytes());
            self.script.extend_from_slice(data);
        }
        self
    }

    pub fn add_i64(&mut self, val: i64) -> &mut Self {
        match val {
            0 => self.add_op(OpCode::OpFalse),
            -1 => self.add_op(OpCode::Op1Negate),
            1..=16 => {
                let op_byte = 0x51 + (val - 1) as u8;
                self.script.push(op_byte);
                self
            }
            _ => {
                let encoded = crate::data_stack::encode_script_num(val);
                self.add_data(&encoded)
            }
        }
    }

    pub fn add_p2pk(&mut self, pubkey: &[u8]) -> &mut Self {
        self.add_data(pubkey).add_op(OpCode::OpCheckSig)
    }

    pub fn add_p2sh(&mut self, script_hash: &[u8]) -> &mut Self {
        self.add_op(OpCode::OpBlake2b)
            .add_data(script_hash)
            .add_op(OpCode::OpEqual)
    }

    pub fn drain(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.script)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.script
    }
}
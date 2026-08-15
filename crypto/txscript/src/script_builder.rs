use crate::opcodes::macros::Opcode;

#[derive(Default, Debug, Clone)]
pub struct ScriptBuilder {
    script: Vec<u8>,
}

impl ScriptBuilder {
    pub fn new() -> Self {
        Self { script: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            script: Vec::with_capacity(capacity),
        }
    }

    pub fn add_op(&mut self, op: Opcode) -> &mut Self {
        self.script.push(op.into());
        self
    }

    pub fn add_ops(&mut self, ops: &[Opcode]) -> &mut Self {
        for &op in ops {
            self.add_op(op);
        }
        self
    }

    pub fn add_data(&mut self, data: &[u8]) -> &mut Self {
        let len = data.len();
        if len == 0 {
            self.add_op(Opcode::OpFalse);
            return self;
        }

        if len < 0x4c {
            self.script.push(len as u8);
        } else if len <= 0xff {
            self.add_op(Opcode::OpPushData1);
            self.script.push(len as u8);
        } else if len <= 0xffff {
            self.add_op(Opcode::OpPushData2);
            self.script.extend_from_slice(&(len as u16).to_le_bytes());
        } else {
            self.add_op(Opcode::OpPushData4);
            self.script.extend_from_slice(&(len as u32).to_le_bytes());
        }

        self.script.extend_from_slice(data);
        self
    }

    pub fn add_i64(&mut self, val: i64) -> &mut Self {
        if val == 0 {
            self.add_op(Opcode::OpFalse);
        } else if (1..=16).contains(&val) {
            let op = match val {
                1 => Opcode::OpTrue,
                2 => Opcode::Op2,
                3 => Opcode::Op3,
                4 => Opcode::Op4,
                5 => Opcode::Op5,
                6 => Opcode::Op6,
                7 => Opcode::Op7,
                8 => Opcode::Op8,
                9 => Opcode::Op9,
                10 => Opcode::Op10,
                11 => Opcode::Op11,
                12 => Opcode::Op12,
                13 => Opcode::Op13,
                14 => Opcode::Op14,
                15 => Opcode::Op15,
                16 => Opcode::Op16,
                _ => unreachable!(),
            };
            self.add_op(op);
        } else if val == -1 {
            self.add_op(Opcode::Op1Negate);
        } else {
            // Encode as Bitcoin/Kaspa script number (sign bit in highest byte)
            let mut res = Vec::new();
            let mut temp = val.unsigned_abs();
            while temp > 0 {
                res.push((temp & 0xff) as u8);
                temp >>= 8;
            }
            if (*res.last().unwrap() & 0x80) != 0 {
                res.push(if val < 0 { 0x80 } else { 0x00 });
            } else if val < 0 {
                let last = res.len() - 1;
                res[last] |= 0x80;
            }
            self.add_data(&res);
        }
        self
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.script
    }

    pub fn drain(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.script)
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.script
    }
}

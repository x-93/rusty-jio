// opcodes.rs
//
// Bitcoin / Kaspa compatible Opcode constants and definitions.

pub mod codes {
    // Push value
    pub const OP_0: u8 = 0x00;
    pub const OP_FALSE: u8 = 0x00;
    pub const OP_PUSHDATA1: u8 = 0x4c;
    pub const OP_PUSHDATA2: u8 = 0x4d;
    pub const OP_PUSHDATA4: u8 = 0x4e;
    pub const OP_1NEGATE: u8 = 0x4f;
    pub const OP_1: u8 = 0x51;
    pub const OP_TRUE: u8 = 0x51;
    pub const OP_2: u8 = 0x52;
    pub const OP_3: u8 = 0x53;
    pub const OP_4: u8 = 0x54;
    pub const OP_5: u8 = 0x55;
    pub const OP_6: u8 = 0x56;
    pub const OP_7: u8 = 0x57;
    pub const OP_8: u8 = 0x58;
    pub const OP_9: u8 = 0x59;
    pub const OP_10: u8 = 0x5a;
    pub const OP_11: u8 = 0x5b;
    pub const OP_12: u8 = 0x5c;
    pub const OP_13: u8 = 0x5d;
    pub const OP_14: u8 = 0x5e;
    pub const OP_15: u8 = 0x5f;
    pub const OP_16: u8 = 0x60;

    // Flow control
    pub const OP_NOP: u8 = 0x61;
    pub const OP_VERIFY: u8 = 0x69;
    pub const OP_RETURN: u8 = 0x6a;

    // Stack
    pub const OP_DUP: u8 = 0x76;
    pub const OP_DROP: u8 = 0x75;
    pub const OP_SWAP: u8 = 0x7c;

    // Splice / Comparison
    pub const OP_EQUAL: u8 = 0x87;
    pub const OP_EQUALVERIFY: u8 = 0x88;

    // Crypto
    pub const OP_SHA256: u8 = 0xa8;
    pub const OP_BLAKE2B: u8 = 0xaa;
    pub const OP_CHECKSIG: u8 = 0xac;
    pub const OP_CHECKSIGVERIFY: u8 = 0xad;
    pub const OP_CHECKSIG_ECDSA: u8 = 0xab;
}


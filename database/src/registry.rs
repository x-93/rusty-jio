//! Prefix registry preventing collision among consensus database stores.

pub struct DatabaseStorePrefixes;

impl DatabaseStorePrefixes {
    pub const HEADERS: &'static [u8] = b"hdr";
    pub const HEADER_SELECTED_TIP: &'static [u8] = b"hst";
    pub const BLOCK_TRANSACTIONS: &'static [u8] = b"txs";
    pub const GHOSTDAG: &'static [u8] = b"gdg";
    pub const REACHABILITY: &'static [u8] = b"rch";
    pub const RELATIONS: &'static [u8] = b"rel";
    pub const STATUSES: &'static [u8] = b"sts";
    pub const TIPS: &'static [u8] = b"tip";
    pub const SELECTED_CHAIN: &'static [u8] = b"sch";
    pub const UTXO_SET: &'static [u8] = b"utx";
    pub const UTXO_DIFFS: &'static [u8] = b"udf";
    pub const UTXO_MULTISETS: &'static [u8] = b"ums";
    pub const PRUNING: &'static [u8] = b"prn";
    pub const PAST_PRUNING_POINTS: &'static [u8] = b"ppp";
    pub const PRUNING_UTXOSET: &'static [u8] = b"pus";
    pub const VIRTUAL_STATE: &'static [u8] = b"vst";
    pub const DAA: &'static [u8] = b"daa";
    pub const BLOCK_DEPTH: &'static [u8] = b"dep";
    pub const ACCEPTANCE_DATA: &'static [u8] = b"acd";
}

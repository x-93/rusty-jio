use enum_primitive_derive::Primitive;

/// We use `u8::MAX` which is never a valid block level. Also note that through
/// the [`DatabaseStorePrefixes`] enum we make sure it is not used as a prefix as well
pub const SEPARATOR: u8 = u8::MAX;

const LOOKUP_BYTES: [[u8; 1]; 256] = {
    let mut arr = [[0u8; 1]; 256];
    let mut i = 0;
    while i < 256 {
        arr[i] = [i as u8];
        i += 1;
    }
    arr
};

#[derive(Primitive, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DatabaseStorePrefixes {
    // ---- Consensus ----
    AcceptanceData = 1,
    BlockTransactions = 2,
    NonDaaMergeset = 3,
    BlockDepth = 4,
    Ghostdag = 5,
    GhostdagCompact = 6,
    HeadersSelectedTip = 7,
    Headers = 8,
    HeadersCompact = 9,
    PastPruningPoints = 10,
    PruningUtxoset = 11,
    PruningUtxosetPosition = 12,
    PruningPoint = 13,
    HistoryRoot = 14,
    Reachability = 15,
    ReachabilityReindexRoot = 16,
    ReachabilityRelations = 17,
    RelationsParents = 18,
    RelationsChildren = 19,
    ChainHashByIndex = 20,
    ChainIndexByHash = 21,
    ChainHighestIndex = 22,
    Statuses = 23,
    Tips = 24,
    UtxoDiffs = 25,
    UtxoMultisets = 26,
    VirtualUtxoset = 27,
    VirtualState = 28,
    DaaScore = 29,

    // ---- Decomposed reachability stores ----
    ReachabilityTreeChildren = 30,
    ReachabilityFutureCoveringSet = 31,

    // ---- Metadata ----
    MultiConsensusMetadata = 124,
    ConsensusEntries = 125,

    // ---- Components ----
    Addresses = 128,
    BannedAddresses = 129,

    // ---- Indexes ----
    UtxoIndex = 192,
    UtxoIndexTips = 193,
    CirculatingSupply = 194,

    // ---- Separator ----
    /// Reserved as a separator
    Separator = SEPARATOR,
}

impl DatabaseStorePrefixes {
    #[inline(always)]
    pub const fn as_ref(&self) -> &'static [u8] {
        &LOOKUP_BYTES[*self as usize]
    }
}

impl From<DatabaseStorePrefixes> for Vec<u8> {
    fn from(value: DatabaseStorePrefixes) -> Self {
        [value as u8].to_vec()
    }
}

impl From<DatabaseStorePrefixes> for u8 {
    fn from(value: DatabaseStorePrefixes) -> Self {
        value as u8
    }
}

impl AsRef<[u8]> for DatabaseStorePrefixes {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl IntoIterator for DatabaseStorePrefixes {
    type Item = u8;
    type IntoIter = <[u8; 1] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        [self as u8].into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_ref() {
        let prefix = DatabaseStorePrefixes::AcceptanceData;
        assert_eq!(&[prefix as u8], prefix.as_ref());
        assert_eq!(
            size_of::<u8>(),
            size_of::<DatabaseStorePrefixes>(),
            "DatabaseStorePrefixes is expected to have the same memory layout of u8"
        );
    }
}

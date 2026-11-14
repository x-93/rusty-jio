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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::AcceptanceData),
            2 => Some(Self::BlockTransactions),
            3 => Some(Self::NonDaaMergeset),
            4 => Some(Self::BlockDepth),
            5 => Some(Self::Ghostdag),
            6 => Some(Self::GhostdagCompact),
            7 => Some(Self::HeadersSelectedTip),
            8 => Some(Self::Headers),
            9 => Some(Self::HeadersCompact),
            10 => Some(Self::PastPruningPoints),
            11 => Some(Self::PruningUtxoset),
            12 => Some(Self::PruningUtxosetPosition),
            13 => Some(Self::PruningPoint),
            14 => Some(Self::HistoryRoot),
            15 => Some(Self::Reachability),
            16 => Some(Self::ReachabilityReindexRoot),
            17 => Some(Self::ReachabilityRelations),
            18 => Some(Self::RelationsParents),
            19 => Some(Self::RelationsChildren),
            20 => Some(Self::ChainHashByIndex),
            21 => Some(Self::ChainIndexByHash),
            22 => Some(Self::ChainHighestIndex),
            23 => Some(Self::Statuses),
            24 => Some(Self::Tips),
            25 => Some(Self::UtxoDiffs),
            26 => Some(Self::UtxoMultisets),
            27 => Some(Self::VirtualUtxoset),
            28 => Some(Self::VirtualState),
            29 => Some(Self::DaaScore),

            124 => Some(Self::MultiConsensusMetadata),
            125 => Some(Self::ConsensusEntries),

            128 => Some(Self::Addresses),
            129 => Some(Self::BannedAddresses),

            192 => Some(Self::UtxoIndex),
            193 => Some(Self::UtxoIndexTips),
            194 => Some(Self::CirculatingSupply),

            SEPARATOR => Some(Self::Separator),
            _ => None,
        }
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
            std::mem::size_of::<u8>(),
            std::mem::size_of::<DatabaseStorePrefixes>(),
            "DatabaseStorePrefixes is expected to have the same memory layout of u8"
        );
    }

    #[test]
    fn test_round_trip() {
        let prefix = DatabaseStorePrefixes::Ghostdag;
        let byte: u8 = prefix.into();
        assert_eq!(byte, 5);
        assert_eq!(DatabaseStorePrefixes::from_u8(byte), Some(prefix));
        assert_eq!(DatabaseStorePrefixes::from_u8(254), None);
    }
}

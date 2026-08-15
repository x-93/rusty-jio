use jio_addresses::{Address, AddressVersion, Prefix};
use jio_consensus_core::tx::{ScriptPublicKey, Transaction, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput};

pub struct RothschildWallet {
    pub prefix: Prefix,
    pub address: Address,
}

impl RothschildWallet {
    pub fn new(prefix: Prefix) -> Self {
        let payload = vec![0x11; 32];
        let address = Address::new(prefix.clone(), AddressVersion::PubKey, payload);
        Self { prefix, address }
    }

    pub fn create_test_tx(&self, prev_tx_id: TransactionId, index: u32, amount: u64) -> Transaction {
        let outpoint = TransactionOutpoint::new(prev_tx_id, index);
        let input = TransactionInput::new(outpoint, vec![0x01; 64], 0, 1);
        let spk = ScriptPublicKey::new(0, self.address.payload.clone());
        let output = TransactionOutput::new(amount.saturating_sub(1000), spk);

        Transaction::new(
            0,
            vec![input],
            vec![output],
            0,
            jio_consensus_core::subnets::SUBNETWORK_ID_NATIVE,
            0,
            vec![],
        )
    }
}

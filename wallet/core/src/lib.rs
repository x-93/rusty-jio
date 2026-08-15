pub mod account;
pub mod address;
pub mod balance;
pub mod generator;
pub mod keychain;
pub mod storage;
pub mod tx;
pub mod wallet;

pub use account::*;
pub use address::*;
pub use balance::*;
pub use generator::*;
pub use keychain::*;
pub use storage::*;
pub use tx::*;
pub use wallet::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_addresses::Prefix;
    use jio_consensus_core::tx::{ScriptPublicKey, TransactionOutpoint};
    use jio_consensus_core::utxo::UtxoEntry;
    use jio_hashes::Hash;

    #[test]
    fn test_wallet_creation_and_address_generation() {
        let wallet = Wallet::create_random("test_wallet".to_string(), Prefix::Devnet, "").expect("wallet created");
        let account = wallet.default_account().expect("account 0");
        let addr = account.receive_address().expect("receive addr");
        assert_eq!(addr.address.prefix, Prefix::Devnet);
    }

    #[test]
    fn test_transaction_generator() {
        let spk = ScriptPublicKey::from_vec(0, vec![1, 2, 3]);
        let change_spk = ScriptPublicKey::from_vec(0, vec![4, 5, 6]);
        let op = TransactionOutpoint::new(Hash::default(), 0);
        let entry = UtxoEntry::new(100_000_000, spk.clone(), 0, false);

        let tx = TransactionGenerator::create_unsigned_tx(
            &[(op, entry)],
            spk,
            50_000_000,
            change_spk,
            1_000,
        )
        .expect("unsigned tx created");

        assert_eq!(tx.outputs.len(), 2);
        assert_eq!(tx.outputs[0].value, 50_000_000);
        assert_eq!(tx.outputs[1].value, 49_999_000);
    }
}

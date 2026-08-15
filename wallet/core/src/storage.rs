use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletStorage {
    pub encrypted_mnemonic: String,
    pub filename: String,
}

impl WalletStorage {
    pub fn new(filename: String, encrypted_mnemonic: String) -> Self {
        Self {
            filename,
            encrypted_mnemonic,
        }
    }
}

use crate::derivation::DerivationPath;
use crate::keypair::KeyPair;
use jio_bip32::extended_key::ExtendedPrivateKey;

pub struct XPrv {
    pub inner: ExtendedPrivateKey,
}

impl XPrv {
    pub fn new(inner: ExtendedPrivateKey) -> Self {
        Self { inner }
    }

    pub fn derive_path(&self, path: &DerivationPath) -> Result<KeyPair, String> {
        let mut curr = self.inner.clone();
        for child_num in path.to_child_numbers() {
            curr = curr.derive_child(child_num)?;
        }
        Ok(KeyPair::from_secret_key(&curr.secret_key))
    }
}

use jio_bip32::extended_key::ExtendedPublicKey;

pub struct XPub {
    pub inner: ExtendedPublicKey,
}

impl XPub {
    pub fn new(inner: ExtendedPublicKey) -> Self {
        Self { inner }
    }
}

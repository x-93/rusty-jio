use jio_addresses::{Address, AddressVersion, Prefix as AddrPrefix};
use secp256k1::{Keypair, PublicKey, Secp256k1, SecretKey, XOnlyPublicKey};

#[derive(Clone)]
pub struct KeyPair {
    keypair: Keypair,
}

impl KeyPair {
    pub fn from_secret_key(secret_key: &SecretKey) -> Self {
        let secp = Secp256k1::new();
        let keypair = Keypair::from_secret_key(&secp, secret_key);
        Self { keypair }
    }

    pub fn public_key(&self) -> PublicKey {
        self.keypair.public_key()
    }

    pub fn x_only_public_key(&self) -> (XOnlyPublicKey, secp256k1::Parity) {
        self.keypair.x_only_public_key()
    }

    pub fn to_address(&self, prefix: AddrPrefix) -> Address {
        let (x_only, _) = self.x_only_public_key();
        Address::new(prefix, AddressVersion::PubKey, x_only.serialize().to_vec())
    }
}

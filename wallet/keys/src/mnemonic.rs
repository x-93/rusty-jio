use bip39::{Language, Mnemonic};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JioMnemonic {
    phrase: String,
}

impl JioMnemonic {
    pub fn random(word_count: usize) -> Result<Self, String> {
        let mnemonic = Mnemonic::generate_in(Language::English, word_count).map_err(|e| e.to_string())?;
        Ok(Self {
            phrase: mnemonic.to_string(),
        })
    }

    pub fn from_phrase(phrase: &str) -> Result<Self, String> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, phrase).map_err(|e| e.to_string())?;
        Ok(Self {
            phrase: mnemonic.to_string(),
        })
    }

    pub fn to_seed(&self, password: &str) -> [u8; 64] {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, &self.phrase).expect("valid mnemonic");
        mnemonic.to_seed(password)
    }

    pub fn phrase(&self) -> &str {
        &self.phrase
    }
}

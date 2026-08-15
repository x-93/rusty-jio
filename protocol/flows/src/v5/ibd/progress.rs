pub struct IbdProgress {
    pub downloaded_headers: u64,
    pub downloaded_blocks: u64,
}

impl IbdProgress {
    pub fn new() -> Self {
        Self {
            downloaded_headers: 0,
            downloaded_blocks: 0,
        }
    }
}

impl Default for IbdProgress {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WrpcResolver {
    pub default_url: String,
}

impl WrpcResolver {
    pub fn new(default_url: String) -> Self {
        Self { default_url }
    }
}

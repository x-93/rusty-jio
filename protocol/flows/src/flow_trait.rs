use async_trait::async_trait;

#[async_trait]
pub trait Flow: Send + Sync {
    async fn start(&mut self) -> Result<(), String>;
}

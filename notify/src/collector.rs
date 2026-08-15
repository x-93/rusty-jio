use async_trait::async_trait;

#[async_trait]
pub trait Collector: Send + Sync {
    async fn start(&mut self) -> Result<(), String>;
}

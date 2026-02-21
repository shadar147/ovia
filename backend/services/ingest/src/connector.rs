use async_trait::async_trait;

#[derive(Debug)]
pub struct SyncResult {
    pub source: String,
    pub upserted: usize,
    #[allow(dead_code)]
    pub skipped: usize,
    pub errors: usize,
}

#[async_trait]
pub trait Connector: Send + Sync {
    #[allow(dead_code)]
    fn source_name(&self) -> &str;
    async fn sync(&self) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>>;
}

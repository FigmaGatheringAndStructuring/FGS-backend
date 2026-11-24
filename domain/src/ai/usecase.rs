use crate::ai::model::{AiRequest, AiResult};
use async_trait::async_trait;

#[async_trait]
pub trait AiUseCase: Send + Sync {
    async fn get_result(&self, request: &AiRequest) -> anyhow::Result<AiResult>;
}

use async_trait::async_trait;
use rust_proto::services::Metadata;

use crate::errors::CheckedError;

#[async_trait]
pub trait PayloadRetriever<T> {
    type Message;
    type Error: CheckedError;
    async fn retrieve_event(
        &mut self,
        msg: &Self::Message,
    ) -> Result<Option<(Metadata, T)>, Self::Error>;
}

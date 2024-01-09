use std::{fmt, sync::Arc};

use anyhow::Result;

use crate::{
    coordinator_client::CoordinatorClient,
    grpc_helper::GrpcHelper,
    indexify_coordinator::{CreateIndexRequest, Index},
    persistence::{ExtractedAttributes, Repository},
};

pub struct AttributeIndexManager {
    repository: Arc<Repository>,
    coordinator_client: Arc<CoordinatorClient>,
}

impl fmt::Debug for AttributeIndexManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttributeIndexManager").finish()
    }
}

impl AttributeIndexManager {
    pub fn new(repository: Arc<Repository>, coordinator_client: Arc<CoordinatorClient>) -> Self {
        Self {
            repository,
            coordinator_client,
        }
    }

    pub async fn create_index(
        &self,
        repository: &str,
        index_name: &str,
        extractor: &str,
        extractor_binding: &str,
        schema: serde_json::Value,
    ) -> Result<String> {
        let index = CreateIndexRequest {
            index: Some(Index {
                name: index_name.to_string(),
                table_name: "structured_store".to_string(),
                repository: repository.to_string(),
                schema: schema.to_string(),
                extractor: extractor.to_string(),
                extractor_binding: extractor_binding.to_string(),
            }),
        };
        let req = GrpcHelper::into_req(index);
        let _resp = self
            .coordinator_client
            .get()
            .await?
            .create_index(req)
            .await?;
        Ok(index_name.to_string())
    }

    pub async fn add_index(
        &self,
        repository: &str,
        index_name: &str,
        extracted_attributes: ExtractedAttributes,
    ) -> Result<()> {
        self.repository
            .add_attributes(repository, index_name, extracted_attributes)
            .await?;
        Ok(())
    }

    pub async fn get_attributes(
        &self,
        repository: &str,
        index_name: &str,
        content_id: Option<&String>,
    ) -> Result<Vec<ExtractedAttributes>> {
        let extracted_attributes = self
            .repository
            .get_extracted_attributes(repository, index_name, content_id)
            .await?;
        Ok(extracted_attributes)
    }
}

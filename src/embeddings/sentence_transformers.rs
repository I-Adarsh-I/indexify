use std::sync::{mpsc, Arc};
use std::thread;

use async_trait::async_trait;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};

use super::server_config::{self, EmbeddingModelKind};
use super::{EmbeddingGenerator, EmbeddingGeneratorError};
use std::collections::HashMap;

type Message = (
    String,
    Vec<String>,
    oneshot::Sender<Result<Vec<Vec<f32>>, EmbeddingGeneratorError>>,
);
pub struct SentenceTransformerModels {
    sender: mpsc::SyncSender<Message>,
}

impl SentenceTransformerModels {
    pub fn new(
        models_to_load: Vec<server_config::EmbeddingModel>,
    ) -> Result<Arc<dyn EmbeddingGenerator + Sync + Send>, EmbeddingGeneratorError> {
        let (sender, receiver) = mpsc::sync_channel(100);
        thread::spawn(move || {
            if let Err(err) = Self::runner(receiver, models_to_load) {
                tracing::error!("embedding generator runner exited with error: {}", err);
            }
        });
        Ok(Arc::new(SentenceTransformerModels { sender }))
    }

    fn runner(
        receiver: mpsc::Receiver<Message>,
        models_to_load: Vec<server_config::EmbeddingModel>,
    ) -> Result<(), EmbeddingGeneratorError> {
        let mut models: HashMap<String, SentenceEmbeddingsModel> = HashMap::new();
        for model in &models_to_load {
            match &model.model_kind {
                EmbeddingModelKind::AllMiniLmL12V2 => {
                    let model = SentenceEmbeddingsBuilder::remote(
                        SentenceEmbeddingsModelType::AllMiniLmL12V2,
                    )
                    .create_model()
                    .map_err(|e| EmbeddingGeneratorError::ModelLoadingError(e.to_string()))?;
                    models.insert(EmbeddingModelKind::AllMiniLmL12V2.to_string(), model);
                }
                EmbeddingModelKind::AllMiniLmL6V2 => {
                    let model = SentenceEmbeddingsBuilder::remote(
                        SentenceEmbeddingsModelType::AllMiniLmL6V2,
                    )
                    .create_model()
                    .map_err(|e| EmbeddingGeneratorError::ModelLoadingError(e.to_string()))?;
                    models.insert(EmbeddingModelKind::AllMiniLmL6V2.to_string(), model);
                }
                EmbeddingModelKind::AllMPNetBaseV2 => {
                    let model =
                        SentenceEmbeddingsBuilder::local("/home/diptanuc/Src/all-mpnet-base-v2/")
                            .create_model()
                            .map_err(|e| {
                                EmbeddingGeneratorError::ModelLoadingError(e.to_string())
                            })?;
                    models.insert(EmbeddingModelKind::AllMPNetBaseV2.to_string(), model);
                }
                EmbeddingModelKind::T5Base => {
                    let model = SentenceEmbeddingsBuilder::remote(
                        SentenceEmbeddingsModelType::SentenceT5Base,
                    )
                    .create_model()
                    .map_err(|e| EmbeddingGeneratorError::ModelLoadingError(e.to_string()))?;

                    models.insert(EmbeddingModelKind::T5Base.to_string(), model);
                }
                _ => {
                    return Err(EmbeddingGeneratorError::InternalError(
                        "unknown model kind".into(),
                    ));
                }
            }
        }
        for (model_name, inputs, sender) in receiver.iter() {
            let model = models.get(&model_name);
            if model.is_none() {
                let _ = sender.send(Err(EmbeddingGeneratorError::ModelNotFound(model_name)));
                continue;
            }
            let result = model
                .unwrap()
                .encode(&inputs)
                .map_err(|e| EmbeddingGeneratorError::ModelError(e.to_string()));
            let _ = sender.send(result);
        }
        Ok(())
    }
}

#[async_trait]
impl EmbeddingGenerator for SentenceTransformerModels {
    async fn generate_embeddings(
        &self,
        texts: Vec<String>,
        model: String,
    ) -> Result<Vec<Vec<f32>>, EmbeddingGeneratorError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.sender.send((model, texts, tx));
        match rx.await {
            Ok(result) => result,
            Err(_) => Err(EmbeddingGeneratorError::InternalError(
                "channel closed unexpectedly".into(),
            )),
        }
    }

    fn dimensions(&self, model: String) -> Result<i16, EmbeddingGeneratorError> {
        match model.as_str() {
            "all-minilm-l12-v2" => Ok(384),
            "all-minilm-l6-v2" => Ok(384),
            "all-mpnet-base-v2" => Ok(768),
            "all-distilroberta-v1" => Ok(768),
            "t5-base" => Ok(1536),
            _ => Err(EmbeddingGeneratorError::ModelNotFound(model)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_embeddings_all_mini_lm_l12v2() {
        use super::*;
        use server_config::DeviceKind;
        use server_config::EmbeddingModelKind::AllMiniLmL12V2;

        let inputs = vec![
            "Hello, world!".to_string(),
            "Hello, NBA!".to_string(),
            "Hello, NFL!".to_string(),
        ];
        let embedding_generator =
            SentenceTransformerModels::new(vec![server_config::EmbeddingModel {
                model_kind: AllMiniLmL12V2,
                device_kind: DeviceKind::Cpu,
            }])
            .unwrap();
        let embeddings = embedding_generator
            .generate_embeddings(inputs, "all-minilm-l12-v2".into())
            .await
            .unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 384);
    }

    #[tokio::test]
    async fn test_local_mpnet_basev2() {
        use super::*;
        use server_config::DeviceKind;
        use server_config::EmbeddingModelKind::AllMPNetBaseV2;

        let inputs = vec![
            "Hello, world!".to_string(),
            "Hello, NBA!".to_string(),
            "Hello, NFL!".to_string(),
        ];
        let embedding_generator =
            SentenceTransformerModels::new(vec![server_config::EmbeddingModel {
                model_kind: AllMPNetBaseV2,
                device_kind: DeviceKind::Cpu,
            }])
            .unwrap();

        println!("LOADED MODEL 1111");
        let embeddings = embedding_generator
            .generate_embeddings(inputs, "all-mpnet-base-v2".into())
            .await
            .unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 384);
    }
}

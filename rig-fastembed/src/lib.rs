use std::sync::Arc;

pub use fastembed::EmbeddingModel as FastembedModel;
use fastembed::{
    InitOptions, InitOptionsUserDefined, ModelInfo, TextEmbedding, UserDefinedEmbeddingModel,
};
use rig::{
    embeddings::{self, EmbeddingError, EmbeddingsBuilder},
    Embed,
};

#[derive(Clone)]
pub struct Client;

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// Create a new Fastembed client.
    pub fn new() -> Self {
        Self
    }

    /// Create an embedding model with the given name.
    /// Note: default embedding dimension of 0 will be used if model is not known.
    /// If this is the case, it's better to use function `embedding_model_with_ndims`
    ///
    /// # Example
    /// ```
    /// use rig_fastembed::{Client, FastembedModel};
    ///
    /// // Initialize the OpenAI client
    /// let fastembed_client = Client::new("your-open-ai-api-key");
    ///
    /// let embedding_model = fastembed_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
    /// ```
    pub fn embedding_model(&self, model: &FastembedModel) -> EmbeddingModel {
        let ndims = fetch_model_ndims(model);

        EmbeddingModel::new(model, ndims)
    }

    /// Create an embedding builder with the given embedding model.
    ///
    /// # Example
    /// ```
    /// use rig_fastembed::{Client, FastembedModel};
    ///
    /// // Initialize the Fastembed client
    /// let fastembed_client = Client::new();
    ///
    /// let embeddings = fastembed_client.embeddings(FastembedModel::AllMiniLML6V2Q)
    ///     .simple_document("doc0", "Hello, world!")
    ///     .simple_document("doc1", "Goodbye, world!")
    ///     .build()
    ///     .await
    ///     .expect("Failed to embed documents");
    /// ```
    pub fn embeddings<D: Embed>(
        &self,
        model: &fastembed::EmbeddingModel,
    ) -> EmbeddingsBuilder<EmbeddingModel, D> {
        EmbeddingsBuilder::new(self.embedding_model(model))
    }
}

#[derive(Clone)]
pub struct EmbeddingModel {
    embedder: Arc<TextEmbedding>,
    pub model: FastembedModel,
    ndims: usize,
}

impl EmbeddingModel {
    pub fn new(model: &fastembed::EmbeddingModel, ndims: usize) -> Self {
        let embedder = Arc::new(
            TextEmbedding::try_new(
                InitOptions::new(model.to_owned()).with_show_download_progress(true),
            )
            .unwrap(),
        );

        Self {
            embedder,
            model: model.to_owned(),
            ndims,
        }
    }

    pub fn new_from_user_defined(
        user_defined_model: UserDefinedEmbeddingModel,
        ndims: usize,
        model_info: &ModelInfo<FastembedModel>,
    ) -> Self {
        let fastembed_embedding_model = TextEmbedding::try_new_from_user_defined(
            user_defined_model,
            InitOptionsUserDefined::default(),
        )
        .unwrap();

        let embedder = Arc::new(fastembed_embedding_model);

        Self {
            embedder,
            model: model_info.model.to_owned(),
            ndims,
        }
    }
}

impl embeddings::EmbeddingModel for EmbeddingModel {
    const MAX_DOCUMENTS: usize = 1024;

    fn ndims(&self) -> usize {
        self.ndims
    }

    async fn embed_texts(
        &self,
        documents: impl IntoIterator<Item = String>,
    ) -> Result<Vec<embeddings::Embedding>, EmbeddingError> {
        let documents_as_strings: Vec<String> = documents.into_iter().collect();

        let documents_as_vec = self
            .embedder
            .embed(documents_as_strings.clone(), None)
            .map_err(|err| EmbeddingError::ProviderError(err.to_string()))?;

        let docs = documents_as_strings
            .into_iter()
            .zip(documents_as_vec)
            .map(|(document, embedding)| embeddings::Embedding {
                document,
                vec: embedding.into_iter().map(|f| f as f64).collect(),
            })
            .collect::<Vec<embeddings::Embedding>>();

        Ok(docs)
    }
}

/// As seen on the text embedding model cards file: <https://github.com/Anush008/fastembed-rs/blob/main/src/models/text_embedding.rs>
pub fn fetch_model_ndims(model: &FastembedModel) -> usize {
    match model {
        FastembedModel::AllMiniLML6V2
        | FastembedModel::AllMiniLML6V2Q
        | FastembedModel::AllMiniLML12V2
        | FastembedModel::AllMiniLML12V2Q
        | FastembedModel::BGESmallENV15
        | FastembedModel::BGESmallENV15Q
        | FastembedModel::ParaphraseMLMiniLML12V2Q
        | FastembedModel::ParaphraseMLMiniLML12V2
        | FastembedModel::MultilingualE5Small => 384,
        FastembedModel::BGESmallZHV15 | FastembedModel::ClipVitB32 => 512,
        FastembedModel::BGEBaseENV15
        | FastembedModel::BGEBaseENV15Q
        | FastembedModel::NomicEmbedTextV1
        | FastembedModel::NomicEmbedTextV15
        | FastembedModel::NomicEmbedTextV15Q
        | FastembedModel::ParaphraseMLMpnetBaseV2
        | FastembedModel::MultilingualE5Base
        | FastembedModel::GTEBaseENV15
        | FastembedModel::GTEBaseENV15Q
        | FastembedModel::JinaEmbeddingsV2BaseCode => 768,
        FastembedModel::BGELargeENV15
        | FastembedModel::BGELargeENV15Q
        | FastembedModel::MultilingualE5Large
        | FastembedModel::MxbaiEmbedLargeV1
        | FastembedModel::MxbaiEmbedLargeV1Q
        | FastembedModel::GTELargeENV15
        | FastembedModel::GTELargeENV15Q => 1024,
    }
}

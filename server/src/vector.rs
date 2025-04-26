use std::error::Error;
use std::sync::{Arc};
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};
use ollama_rs::Ollama;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{CreateCollectionBuilder, Distance, PointsOperationResponse, PointStruct, QueryPointsBuilder, QueryResponse, UpsertPointsBuilder, VectorParamsBuilder};
use tokio::sync::Mutex;
use shared::{Coordinate, Monument};

const MODEL: &'static str = "aroxima/gte-qwen2-1.5b-instruct";
const COLLECTION: &'static str = "monuments";

pub struct Engine {
    ollama: Ollama,
    qdrant: Qdrant,
}

impl Engine {
    pub async fn new() -> Result<Arc<Mutex<Engine>>, Box<dyn Error>> {
        let qdrant = Qdrant::from_url("http://127.0.0.1:6334").build()?;
        let ollama = Ollama::new("http://127.0.0.1:6334", 11434);

        let engine = Self { qdrant, ollama };
        engine.initialize().await?;

        Ok(Arc::new(Mutex::new(engine)))
    }

    async fn initialize(&self) -> Result<(), Box<dyn Error>> {
        if self.qdrant.collection_exists(COLLECTION).await? == false {
            let input = CreateCollectionBuilder::new(COLLECTION).vectors_config(
                VectorParamsBuilder::new(1536, Distance::Cosine)
            );

            if let Err(info) = self.qdrant.create_collection(input).await {
                println!("failed to create collection {:?}", info);
            };
        }

        if let Err(_) = self.ollama.show_model_info(MODEL.into()).await {
            println!("downloading model: {:?}", MODEL);

            if let Err(error) = self.ollama.pull_model(MODEL.into(), false).await {
                panic!("failed to pull ollama model: {:?}", error);
            };

            println!("downloaded successful");
        }

        Ok(())
    }

    pub async fn add_monument(&self, monument: &Monument) -> Result<PointsOperationResponse, Box<dyn Error>> {
        let mut points = vec![];

        let embeddings = self.generate_embedding(monument.description.as_str()).await?;

        points.push(PointStruct::new(0, embeddings, [("id", monument.id.to_string().into())]));

        Ok(self.qdrant
            .upsert_points(UpsertPointsBuilder::new(COLLECTION, points).wait(true))
            .await?)
    }

    pub async fn search(&self, input: &str) -> Result<QueryResponse, Box<dyn Error>> {
        let embeddings = self.generate_embedding(input).await?;

        Ok(self.qdrant
            .query(QueryPointsBuilder::new(COLLECTION).query(embeddings).with_payload(true))
            .await?)
    }

    pub async fn generate_embedding(&self, input: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let input = EmbeddingsInput::Single(input.into());
        let request = GenerateEmbeddingsRequest::new(MODEL.into(), input);

        Ok(self.ollama.generate_embeddings(request).await?.embeddings[0].to_owned())
    }
}

#[tokio::test]
async fn start() -> Result<(), Box<dyn Error>> {
    let engine = Engine::new().await?;

    let monument = Monument {
        id: 2,
        position: Coordinate { x: 0, y: 0 },
        asset: "cool.png".into(),
        description: "a funny dog walking on the street".into(),
        under_construction: true,
    };

    // let points = vec![monument];
    // engine.add_points(points).await?;
    //
    // let response = engine.search("dogs").await?;
    //
    // println!("{:?}", response);

    Ok(())
}

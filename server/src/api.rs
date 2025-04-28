use std::collections::HashMap;

use axum::extract::{Multipart, State};
use axum::http::Method;
use axum::Router;
use axum::routing::post;
use tokio::fs::OpenOptions;
use tower_http::cors::{Any, CorsLayer};

use shared::{Monument, SystemMessages};

use crate::manager::Manager;
use crate::world::World;

pub async fn build_server(manager: Manager, world: World) {
    let app = Router::new()
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .route("/generation", post(handle))
        .with_state((manager, world))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any)
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("api server starting at {}", "http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn handle(State((manager, world)): State<(Manager, World)>, multipart: Multipart) {
    let (id, asset) = handle_payload(multipart).await.unwrap();

    world.complete_monument(id, &asset).await;
    manager.broadcast(SystemMessages::MonumentCompleted { id, asset }).await;
}

async fn handle_payload(mut multipart: Multipart) -> Result<(u32, String), Box<dyn std::error::Error>> {
    let mut map: HashMap<String, String> = HashMap::new();

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        match name.as_str() {
            name if name == "file" => {
                let id = uuid::Uuid::new_v4();
                tokio::fs::create_dir_all("./assets/monuments").await.unwrap();
                tokio::fs::write(format!("./assets/monuments/{}.png", id), data).await.unwrap();
                map.insert(name.to_string(), format!("{}/assets/monuments/{}.png", env!("API_SERVER_ADDRESS"), id));
            }
            name if name == "prompt_id" => {
                map.insert(name.to_string(), String::from_utf8_lossy(&data).to_string());
            }
            _ => continue,
        }
    }

    Ok((
        map.get("prompt_id").unwrap().parse::<u32>().unwrap(),
        map.get("file").unwrap().to_string(),
    ))
}

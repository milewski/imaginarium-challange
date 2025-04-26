use crate::{Manager, World};
use axum::extract::{Multipart, State};
use axum::http::Method;
use axum::routing::post;
use axum::Router;
use shared::SystemMessages;
use std::collections::HashMap;
use tower_http::cors::{Any, CorsLayer};

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
                tokio::fs::write(format!("./assets/monuments/{}.png", id), data).await.unwrap();
                map.insert(name.to_string(), format!("http://127.0.0.1:3000/assets/monuments/{}.png", id));
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

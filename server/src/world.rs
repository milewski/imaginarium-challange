use std::collections::HashMap;
use std::sync::Arc;

use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use shared::{Coordinate, Monument, PlayerData, PlayerId};

#[derive(Default, Clone)]
pub struct World {
    inner: Arc<Mutex<HashMap<PlayerId, PlayerData>>>,
    monuments: Arc<Mutex<HashMap<u32, Monument>>>,
}

impl World {
    pub async fn get(&self, id: PlayerId) -> Option<PlayerData> {
        self.inner.lock().await.get(&id).cloned()
    }

    pub async fn players(&self) -> Vec<PlayerData> {
        self.inner.lock().await.values().cloned().collect()
    }

    pub async fn monuments(&self) -> Vec<Monument> {
        self.monuments.lock().await.values().cloned().collect()
    }

    pub async fn add_monument(&self, monument: Monument) {
        self.monuments.lock().await.insert(monument.id, monument);
    }

    pub async fn complete_monument(&self, id: u32, asset: &str) {
        let mut monuments = self.monuments.lock().await;

        if let Some(monument) = monuments.get_mut(&id) {
            monument.asset = asset.to_string();
            monument.under_construction = false;

            if let Err(error) = self.cache_monument(&monument).await {
                println!("failed to store monument: {:?}", error)
            }
        }
    }

    pub async fn add(&self, data: PlayerData) {
        self.inner.lock().await.insert(data.id, data);
    }

    pub async fn remove(&self, id: PlayerId) {
        self.inner.lock().await.remove(&id);
    }

    pub async fn update_coordinate(&self, id: PlayerId, coordinate: Coordinate) {
        if let Some(data) = self.inner.lock().await.get_mut(&id) {
            data.position = coordinate
        }
    }

    pub async fn increment_balance(&self, id: PlayerId) -> u32 {
        if let Some(data) = self.inner.lock().await.get_mut(&id) {
            data.balance += 1;
            data.balance
        } else {
            0
        }
    }

    pub async fn decrement_balance_by(&self, id: PlayerId, amount: u32) -> u32 {
        if let Some(data) = self.inner.lock().await.get_mut(&id) {
            data.balance -= amount;
            data.balance
        } else {
            0
        }
    }

    async fn cache_monument(&self, monument: &Monument) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("./assets/monuments.jsonl")
            .await?;

        let line = serde_json::to_string(monument)?;
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;

        Ok(())
    }

    pub async fn restore_monuments_from_cache(&self) -> tokio::io::Result<()> {
        let file = File::open("./assets/monuments.jsonl").await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut monuments = self.monuments.lock().await;

        while let Some(line) = lines.next_line().await? {
            let monument: Monument = serde_json::from_str(&line)?;
            monuments.insert(monument.id, monument);
        }

        Ok(())
    }
}
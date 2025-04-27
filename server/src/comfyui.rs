use serde_json::Value;
use std::collections::HashMap;

const WORKFLOW: &'static str = include_str!("../workflow/isometric.json");

#[derive(Default)]
pub struct ComfyUI;

impl ComfyUI {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate(&self, prompt: &str) -> Result<u32, reqwest::Error> {
        let id = fastrand::u32(..);

        let prompt_id = id.to_string();
        let workflow = WORKFLOW
            .replace("__WEBHOOK_URL__", format!("{}/generation", env!("API_SERVER_ADDRESS")).as_str())
            .replace("1110000111", prompt_id.as_str())
            .replace("__PROMPT__", prompt)
            .replace("__PROMPT_ID__", prompt_id.as_str());

        let map: HashMap<&str, Value> = HashMap::from([
            ("prompt", serde_json::from_str(workflow.as_str()).unwrap())
        ]);

        let _ = reqwest::Client::new()
            .post(format!("{}/api/prompt", env!("COMFYUI_HOST_URL")))
            .json(&map)
            .send()
            .await?;

        Ok(id)
    }
}

#[tokio::test]
async fn test() {
    let comfyui = ComfyUI::default();

    println!("{:?}", comfyui.generate("hello world").await);
}
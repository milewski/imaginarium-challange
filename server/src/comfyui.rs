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
            .replace("__WEBHOOK_URL__", "http://127.0.0.1:3000/generation")
            .replace("1110000111", prompt_id.as_str())
            .replace("__PROMPT__", prompt)
            .replace("__PROMPT_ID__", prompt_id.as_str());

        let map: HashMap<&str, Value> = HashMap::from([
            ("prompt", serde_json::from_str(workflow.as_str()).unwrap())
        ]);

        let _ = reqwest::Client::new()
            .post("http://192.168.50.230:8188/api/prompt")
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
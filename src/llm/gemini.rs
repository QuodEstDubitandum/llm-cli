use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct GEMINI {
    config: GeminiConfig,
    endpoint: &'static str,
}

impl GEMINI {
    fn read_config() -> GeminiConfig {
        let config_path = "/etc/llm_cli_config.json";
        let json_string =
            fs::read_to_string(Path::new(config_path)).expect("--- Could not read config file ---");

        let json_object: serde_json::Value =
            serde_json::from_str(&json_string).expect("--- Could not parse JSON ---");

        let config: GeminiConfig = serde_json::from_value(json_object["gemini"].clone())
            .expect("--- Incorrect Gemini config in the config file ---");

        config
    }
}
impl Default for GEMINI {
    fn default() -> Self {
        let config = GEMINI::read_config();
        GEMINI {
            endpoint: "https://generativelanguage.googleapis.com/v1beta/models/",
            config,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiConfig {
    api_key: String,
    model_version: String,
    max_tokens: u16,
    temperature: f32,
}

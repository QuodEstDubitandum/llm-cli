use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::llm::utils::print_response;

use super::utils::get_config_path;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MistralBody {
    model: String,
    max_tokens: u16,
    temperature: f32,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MistralConfig {
    api_key: String,
    model_name: String,
    max_tokens: u16,
    temperature: f32,
}

#[derive(Clone)]
pub struct MISTRAL {
    config: MistralConfig,
    endpoint: &'static str,
}

impl MISTRAL {
    fn read_config() -> MistralConfig {
        let config_path = get_config_path();
        let json_string =
            fs::read_to_string(config_path).expect("--- Could not read config file ---");

        let json_object: serde_json::Value =
            serde_json::from_str(&json_string).expect("--- Could not parse JSON ---");

        let config: MistralConfig = serde_json::from_value(json_object["mistral"].clone())
            .expect("--- Incorrect Mistral config in the config file ---");

        if config.api_key.is_empty() {
            panic!("--- No Mistral API Key provided ---")
        }

        config
    }
    pub fn parse_args(&mut self, args: Vec<String>) -> String {
        for (i, arg) in args.iter().enumerate() {
            if arg == "$" {
                return args[i + 1..].join(" ");
            }

            if arg.starts_with("-model=") {
                self.config.model_name = arg[7..].to_owned();
                continue;
            }

            if arg.starts_with("-temp=") {
                self.config.temperature = arg[6..]
                    .parse::<f32>()
                    .expect("--- Could not parse temp to a float ---");
                continue;
            }

            if arg.starts_with("-token=") {
                self.config.max_tokens = arg[7..]
                    .parse::<u16>()
                    .expect("--- Could not parse token to a float ---");
                continue;
            }

            panic!("--- Found invalid argument: {arg} ---")
        }
        panic!("--- Missing '$' command ---")
    }
    pub async fn make_request(
        &self,
        prompt: Arc<String>,
        request_number: Arc<AtomicUsize>,
        lock: Arc<Mutex<()>>,
    ) {
        // building the body
        let messages = vec![Message {
            role: String::from("user"),
            content: (*prompt).deref().to_owned(),
        }];
        let body = MistralBody {
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            model: self.config.model_name.clone(),
            messages,
        };
        let json_body = serde_json::to_string(&body).unwrap_or_else(|_| {
            request_number.fetch_sub(1, Ordering::SeqCst);
            panic!("--- Could not stringify Mistral config to JSON ---")
        });

        // making request as well as measuring time taken
        let client_builder = Client::builder().timeout(Duration::from_secs(120));
        let client = client_builder.build().unwrap_or_else(|_| {
            request_number.fetch_sub(1, Ordering::SeqCst);
            panic!("--- Could not create Mistral client ---")
        });
        let req_start = Instant::now();
        let res = client
            .post(self.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "application/json")
            .body(json_body)
            .send()
            .await
            .unwrap_or_else(|_| {
                request_number.fetch_sub(1, Ordering::SeqCst);
                panic!("--- Request to Mistral endpoint failed ---")
            });

        // signal to loading loop to stop/decrement
        request_number.fetch_sub(1, Ordering::SeqCst);
        let req_time = req_start.elapsed();

        // if req successful
        if res.status().is_success() {
            let response_text = res.text().await.unwrap_or_else(|_| {
                request_number.fetch_sub(1, Ordering::SeqCst);
                panic!("--- Failed parsing Mistral response message ---")
            });
            let parsed_response_text: serde_json::Value = serde_json::from_str(&response_text)
                .unwrap_or_else(|_| {
                    request_number.fetch_sub(1, Ordering::SeqCst);
                    panic!("--- Failed parsing Mistral response message ---")
                });
            let response_text: String = serde_json::from_value(
                parsed_response_text["choices"][0]["message"]["content"].clone(),
            )
            .unwrap_or_else(|_| {
                request_number.fetch_sub(1, Ordering::SeqCst);
                panic!("--- Malformed Mistral JSON response ---")
            });

            let _lock = lock.lock().await;
            print_response(&response_text, req_time.as_secs_f64(), 58, "Mistral");
            return;
        }

        // if something went wrong
        let _lock = lock.lock().await;
        panic!(
            "--- Request to Mistral failed with: ---\nStatus Code: {}\nError Message: {}",
            res.status(),
            res.text().await.unwrap_or_else(|_| {
                request_number.fetch_sub(1, Ordering::SeqCst);
                panic!("--- Failed parsing Mistral response message ---")
            })
        )
    }
}
impl Default for MISTRAL {
    fn default() -> Self {
        let config = MISTRAL::read_config();
        MISTRAL {
            endpoint: "https://api.mistral.ai/v1/chat/completions",
            config,
        }
    }
}

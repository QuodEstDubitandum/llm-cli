use llm_cli::llm::claude::CLAUDE;
use llm_cli::llm::gpt::GPT;
use llm_cli::llm::mistral::MISTRAL;
use llm_cli::llm::utils::{loop_loading, parse_prompt};
use std::collections::HashMap;
use std::env;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub enum LlmModel {
    GPT(GPT),
    CLAUDE(CLAUDE),
    MISTRAL(MISTRAL),
}

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args[0] == "llm-cli" {
        args.remove(0);
    }

    let models = get_models(&args[0]);
    let other_args = args[1..].to_vec();
    let all_requests = tokio::spawn(async move {
        handle_model_requests(models, &args[0], other_args).await;
    });

    all_requests.await.unwrap();
}

fn get_models(model_args: &str) -> Vec<LlmModel> {
    let mut models: Vec<LlmModel> = Vec::new();
    let model_args = model_args.split(',');
    for model in model_args {
        match model {
            "gpt" => models.push(LlmModel::GPT(GPT::default())),
            "claude" => models.push(LlmModel::CLAUDE(CLAUDE::default())),
            "mistral" => models.push(LlmModel::MISTRAL(MISTRAL::default())),
            _ => panic!(
                "--- Invalid first argument, choose between 'gpt', 'gemini' and 'claude' or a combination of those seperated by ',' ---"
            ),
        }
    }
    models
}

async fn handle_model_requests(models: Vec<LlmModel>, model_arg: &str, args: Vec<String>) {
    let models_len = models.len();

    // if only one model gets selected
    if models_len == 1 {
        handle_single_request(models[0].clone(), args).await;
        return;
    }

    // if a combination of models get selected
    handle_multiple_requests(model_arg, args, models).await;
}

async fn handle_single_request(model: LlmModel, args: Vec<String>) {
    let request_number_mutex = Arc::new(AtomicUsize::new(1));
    let lock_mutex = Arc::new(Mutex::new(()));
    let lock = Arc::clone(&lock_mutex);
    match model {
        LlmModel::GPT(mut x) => {
            let prompt_mutex = Arc::new(x.parse_args(args));
            let prompt = Arc::clone(&prompt_mutex);
            let request_number = Arc::clone(&request_number_mutex);
            tokio::spawn(async move {
                loop_loading(
                    "Asking GPT",
                    prompt,
                    request_number,
                    Arc::clone(&lock_mutex),
                )
                .await
            });
            let prompt = Arc::clone(&prompt_mutex);
            let request_number = Arc::clone(&request_number_mutex);
            x.make_request(prompt, request_number, lock).await;
        }
        LlmModel::CLAUDE(mut x) => {
            let prompt_mutex = Arc::new(x.parse_args(args));
            let prompt = Arc::clone(&prompt_mutex);
            let request_number = Arc::clone(&request_number_mutex);
            tokio::spawn(async move {
                loop_loading(
                    "Asking Claude",
                    prompt,
                    request_number,
                    Arc::clone(&lock_mutex),
                )
                .await
            });
            let prompt = Arc::clone(&prompt_mutex);
            let request_number = Arc::clone(&request_number_mutex);
            x.make_request(prompt, request_number, lock).await;
        }
        LlmModel::MISTRAL(mut x) => {
            let prompt_mutex = Arc::new(x.parse_args(args));
            let prompt = Arc::clone(&prompt_mutex);
            let request_number = Arc::clone(&request_number_mutex);
            tokio::spawn(async move {
                loop_loading(
                    "Asking Mistral",
                    prompt,
                    request_number,
                    Arc::clone(&lock_mutex),
                )
                .await
            });
            let prompt = Arc::clone(&prompt_mutex);
            let request_number = Arc::clone(&request_number_mutex);
            x.make_request(prompt, request_number, lock).await;
        }
    }
}

async fn handle_multiple_requests(model_arg: &str, args: Vec<String>, models: Vec<LlmModel>) {
    let mut label_map: HashMap<&str, &str> = HashMap::new();
    label_map.insert("gpt", "GPT");
    label_map.insert("claude", "Claude");
    label_map.insert("mistral", "Mistral");

    let prompt_mutex = Arc::new(parse_prompt(args));
    let prompt = Arc::clone(&prompt_mutex);
    let model_labels: Vec<&str> = model_arg
        .split(',')
        .map(|arg| *label_map.get(arg).unwrap())
        .collect();
    let loading_message = format!(
        "Asking {}",
        model_labels
            .join(" and ")
            .replacen(" and ", ", ", models.len() - 2)
    );

    let request_number_mutex = Arc::new(AtomicUsize::new(models.len()));
    let request_number = Arc::clone(&request_number_mutex);
    let lock_mutex = Arc::new(Mutex::new(()));
    let lock = Arc::clone(&lock_mutex);

    let loop_thread = tokio::spawn(async move {
        loop_loading(loading_message.as_str(), prompt, request_number, lock).await
    });

    let mut tokio_threads = vec![loop_thread];
    for model in models {
        let lock = Arc::clone(&lock_mutex);
        let prompt = Arc::clone(&prompt_mutex);
        let request_number = Arc::clone(&request_number_mutex);
        match model {
            LlmModel::GPT(x) => {
                let gpt_thread =
                    tokio::spawn(async move { x.make_request(prompt, request_number, lock).await });
                tokio_threads.push(gpt_thread);
            }
            LlmModel::CLAUDE(x) => {
                let claude_thread =
                    tokio::spawn(async move { x.make_request(prompt, request_number, lock).await });
                tokio_threads.push(claude_thread);
            }
            LlmModel::MISTRAL(x) => {
                let claude_thread =
                    tokio::spawn(async move { x.make_request(prompt, request_number, lock).await });
                tokio_threads.push(claude_thread);
            }
        }
    }

    for thread in tokio_threads {
        thread.await.unwrap();
    }
}

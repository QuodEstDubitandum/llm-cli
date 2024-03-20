use std::{
    io::{stdout, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use tokio::sync::Mutex;

pub fn parse_prompt(args: Vec<String>) -> String {
    if args[0] != "$" {
        panic!("--- Missing '$' command ---")
    }

    args[1..].join(" ")
}

pub fn get_config_path() -> PathBuf {
    let mut config_path = PathBuf::new();
    match std::env::consts::OS {
        "linux" | "macos" => config_path.push("/etc/llm_cli_config.json"),
        _ => {
            panic!("--- Unsupported operating system ---")
        }
    }
    config_path
}

pub fn print_response(response: &String, req_time: f64, divider_number: usize, llm_name: &str) {
    println!(
        "{} {} Response (took {:.2} seconds) {}\n",
        "-".repeat(10),
        llm_name,
        req_time,
        "-".repeat(10)
    );
    println!("{}\n", response);
    println!("{}\n", "-".repeat(divider_number));
}

pub async fn loop_loading(
    loading_text: &str,
    prompt: Arc<String>,
    request_number: Arc<AtomicUsize>,
    lock: Arc<Mutex<()>>,
) {
    // acquire lock to block request threads to not send response message immediately
    let _lock = lock.lock().await;
    let dot_number = 3;
    println!();

    // wait for atomic signals sent upon response finish from request threads
    while request_number.load(Ordering::SeqCst) != 0 {
        print!("{}", loading_text);
        stdout().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
        for _ in 0..dot_number {
            if request_number.load(Ordering::SeqCst) != 0 {
                print!(".");
                stdout().flush().unwrap();
                thread::sleep(Duration::from_secs(1));
            }
        }
        print!("\r{}{}", " ".repeat(50), "\x08".repeat(50));
    }
    print!("{}\n\n", prompt);
}

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Describe a shell task in plain English and get a runnable command back"
)]
struct Cli {
    /// Natural language description of the shell task, e.g. "convert input.mp4 to gif"
    prompt: Vec<String>,

    /// Model to use for the Responses API
    #[arg(long, default_value = "gpt-5.1-codex-max")]
    model: String,

    /// Base URL for the OpenAI API (defaults to api.openai.com)
    #[arg(
        long,
        env = "LLMWRAP_OPENAI_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    api_base: String,
}

#[derive(Serialize)]
struct ResponsesRequest {
    model: String,
    input: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
struct ContentPart {
    #[serde(rename = "type")]
    part_type: String,
    text: String,
}

const SYSTEM_PROMPT: &str = "You translate natural-language requests into a single shell command. \
Respond with only the runnable command, no explanations, no code fences. \
Prefer safe quoting for filenames. If the request is impossible, reply with a brief reason.";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let description = cli.prompt.join(" ");

    if description.trim().is_empty() {
        anyhow::bail!("Please provide a description, e.g. `llmwrap convert video.mp4 to gif`");
    }

    let api_key = std::env::var("LLMWRAP_OPENAI_API_KEY")
        .context("Set LLMWRAP_OPENAI_API_KEY in your environment before running this tool")?;

    let client = Client::builder().build()?;
    let command_text = fetch_command(&client, &api_key, &cli.api_base, &cli.model, &description)
        .await
        .context("Failed to get command from OpenAI Responses API")?;

    println!("\nProposed command:\n{}\n", command_text);

    if !confirm_run()? {
        println!("Aborted by user; command not executed.");
        return Ok(());
    }

    run_command(&command_text)?;
    Ok(())
}

async fn fetch_command(
    client: &Client,
    api_key: &str,
    api_base: &str,
    model: &str,
    user_request: &str,
) -> Result<String> {
    let body = ResponsesRequest {
        model: model.to_string(),
        input: vec![
            Message {
                role: "system".to_string(),
                content: vec![ContentPart {
                    part_type: "input_text".to_string(),
                    text: SYSTEM_PROMPT.to_string(),
                }],
            },
            Message {
                role: "user".to_string(),
                content: vec![ContentPart {
                    part_type: "input_text".to_string(),
                    text: user_request.to_string(),
                }],
            },
        ],
    };

    let url = format!("{}/responses", api_base.trim_end_matches('/'));
    let response = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    // Parse as generic JSON to be resilient to minor schema changes and capture helpful errors
    let body_text = response.text().await?;
    let parsed: Value = serde_json::from_str(&body_text)
        .with_context(|| format!("Failed to decode responses body: {}", body_text))?;

    let raw_text = extract_text(&parsed).context(format!(
        "No text output returned from model. Full body: {}",
        body_text
    ))?;

    Ok(sanitize_command(&raw_text))
}

fn extract_text(value: &Value) -> Option<String> {
    // Primary: output is an array of messages with content
    if let Some(outputs) = value.get("output").and_then(|o| o.as_array()) {
        for msg in outputs {
            if let Some(contents) = msg.get("content").and_then(|c| c.as_array()) {
                for c in contents {
                    if let Some(text) = c.get("text").and_then(|t| t.as_str()) {
                        return Some(text.to_string());
                    }
                }
            }
        }
    }

    // Some payloads may include a single object under "output"
    if let Some(msg) = value.get("output").and_then(|o| o.as_object()) {
        if let Some(contents) = msg.get("content").and_then(|c| c.as_array()) {
            for c in contents {
                if let Some(text) = c.get("text").and_then(|t| t.as_str()) {
                    return Some(text.to_string());
                }
            }
        }
    }

    // Fallback: output_text as string or array
    if let Some(text) = value.get("output_text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }
    if let Some(arr) = value.get("output_text").and_then(|t| t.as_array()) {
        let joined: String = arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if !joined.is_empty() {
            return Some(joined);
        }
    }

    None
}

fn sanitize_command(raw: &str) -> String {
    let first_line = raw
        .lines()
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_matches('`')
        .trim()
        .to_string();
    first_line
}

fn confirm_run() -> Result<bool> {
    print!("Run this command? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let decision = input.trim().to_lowercase();
    Ok(decision == "y" || decision == "yes")
}

fn run_command(command: &str) -> Result<()> {
    println!("Executing: {}", command);
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .context("Failed to spawn shell")?;

    if !status.success() {
        anyhow::bail!("Command exited with status: {}", status);
    }
    Ok(())
}

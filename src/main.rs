use anyhow::{Context, Result};
use clap::Parser;
use csv::Writer;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use std::time::Duration;
use serde::ser::{Serialize, SerializeStruct, Serializer};

const API_KEY: &str = "YOUR_GEMINI_API_KEY";
const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const MAX_EXAMPLES: usize = 1000;
const JSON_FILE: &str = "training_data.json";
const CSV_FILE: &str = "training_data.csv";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Number of examples to generate (max 1000)
    #[arg(short, long)]
    num_examples: usize,

    /// Output directory for generated data
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,
}

#[derive(Debug)]
struct TrainingExample {
    text_input: String,
    output: String,
}

impl Serialize for TrainingExample {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TrainingExample", 2)?;
        state.serialize_field("text_input", &self.text_input)?;
        state.serialize_field("output", &self.output)?;
        state.end()
    }
}

#[derive(Debug)]
struct DataGenerator {
    client: Client,
    output_dir: PathBuf,
    training_data: Vec<TrainingExample>,
    csv_writer: Writer<std::fs::File>,
}

impl DataGenerator {
    async fn new(output_dir: PathBuf) -> Result<Self> {
        // Load existing JSON data if it exists
        let training_data = if output_dir.join(JSON_FILE).exists() {
            let content = fs::read_to_string(output_dir.join(JSON_FILE))
                .await
                .context("Failed to read existing training data")?;
            let json_data: Vec<Value> = serde_json::from_str(&content)
                .context("Failed to parse existing training data")?;
            json_data
                .into_iter()
                .map(|v| TrainingExample {
                    text_input: v["text_input"].as_str().unwrap_or_default().to_string(),
                    output: v["output"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        } else {
            Vec::new()
        };

        // Create or append to CSV file
        let csv_exists = output_dir.join(CSV_FILE).exists();
        let csv_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(output_dir.join(CSV_FILE))
            .context("Failed to open CSV file")?;
        
        let mut csv_writer = Writer::from_writer(csv_file);
        
        // Write header only if file is new
        if !csv_exists {
            csv_writer.write_record(&["text_input", "output"])
                .context("Failed to write CSV header")?;
        }

        Ok(Self {
            client: Client::new(),
            output_dir,
            training_data,
            csv_writer,
        })
    }

    //DEFAULTS TO GEMINI-2.0-FLASH-EXP -- FEEL FREE TO CHANGE
    async fn generate_initial_data(&self) -> Result<String> {
        let url = format!("{}/models/gemini-2.0-flash-exp:generateContent?key={}", GEMINI_BASE_URL, API_KEY);
        
        let mut retries = 0;
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: Duration = Duration::from_secs(5);

        loop {
            let response = self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&json!({
                    "contents": [
                        {
                            "role": "user",
                            "parts": [
                                {
                                    "text": include_str!("prompts/initial_prompt.txt")
                                }
                            ]
                        },
                        {
                            "role": "user",
                            "parts": [
                                {
                                    "text": "INSERT_INPUT_HERE"
                                }
                            ]
                        }
                    ],
                    "generationConfig": {
                        "temperature": 1,
                        "topK": 64,
                        "topP": 0.95,
                        "maxOutputTokens": 8192,
                        "responseMimeType": "application/json"
                    }
                }))
                .send()
                .await
                .context("Failed to send initial request to Gemini API")?;

            if response.status() == 503 {
                if retries >= MAX_RETRIES {
                    anyhow::bail!("Gemini API is overloaded after {} retries", MAX_RETRIES);
                }
                retries += 1;
                tokio::time::sleep(RETRY_DELAY).await;
                continue;
            }

            let json_response = response
                .json::<Value>()
                .await
                .context("Failed to parse initial Gemini API response")?;

            // Check if there's an error in the response
            if let Some(error) = json_response.get("error") {
                if retries >= MAX_RETRIES {
                    anyhow::bail!("Gemini API error after {} retries: {:?}", MAX_RETRIES, error);
                }
                retries += 1;
                tokio::time::sleep(RETRY_DELAY).await;
                continue;
            }

            // Extract the actual generated content from the response
            if let Some(text) = json_response
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str())
            {
                return Ok(text.to_string());
            }

            if retries >= MAX_RETRIES {
                anyhow::bail!("Failed to extract content from response after {} retries", MAX_RETRIES);
            }
            retries += 1;
            tokio::time::sleep(RETRY_DELAY).await;
        }
    }

    async fn analyze_data(&self, initial_data: &str) -> Result<String> {
        let url = format!("{}/models/gemini-2.0-flash-exp:generateContent?key={}", GEMINI_BASE_URL, API_KEY);
        
        let mut retries = 0;
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: Duration = Duration::from_secs(5);

        loop {
            let response = self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&json!({
                    "contents": [
                        {
                            "role": "user",
                            "parts": [
                                {
                                    "text": initial_data
                                }
                            ]
                        },
                        {
                            "role": "user",
                            "parts": [
                                {
                                    "text": "INSERT_INPUT_HERE"
                                }
                            ]
                        }
                    ],
                    "systemInstruction": {
                        "role": "user",
                        "parts": [
                            {
                                "text": include_str!("prompts/system_instruction.txt")
                            }
                        ]
                    },
                    "generationConfig": {
                        "temperature": 1,
                        "topK": 40,
                        "topP": 0.95,
                        "maxOutputTokens": 8192,
                        "responseMimeType": "application/json"
                    }
                }))
                .send()
                .await
                .context("Failed to send analysis request to Gemini API")?;

            if response.status() == 503 {
                if retries >= MAX_RETRIES {
                    anyhow::bail!("Gemini API is overloaded after {} retries", MAX_RETRIES);
                }
                retries += 1;
                tokio::time::sleep(RETRY_DELAY).await;
                continue;
            }

            let json_response = response
                .json::<Value>()
                .await
                .context("Failed to parse analysis Gemini API response")?;

            // Check if there's an error in the response
            if let Some(error) = json_response.get("error") {
                if retries >= MAX_RETRIES {
                    anyhow::bail!("Gemini API error after {} retries: {:?}", MAX_RETRIES, error);
                }
                retries += 1;
                tokio::time::sleep(RETRY_DELAY).await;
                continue;
            }

            // Extract the actual analysis from the response
            if let Some(text) = json_response
                .get("candidates")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("content"))
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.get(0))
                .and_then(|p| p.get("text"))
                .and_then(|t| t.as_str())
            {
                return Ok(text.to_string());
            }

            if retries >= MAX_RETRIES {
                anyhow::bail!("Failed to extract content from response after {} retries", MAX_RETRIES);
            }
            retries += 1;
            tokio::time::sleep(RETRY_DELAY).await;
        }
    }

    async fn save_example(&mut self, initial_data: String, analysis: String) -> Result<()> {
        let example = TrainingExample {
            text_input: initial_data.clone(),
            output: analysis.clone(),
        };
        
        self.training_data.push(example);

        let json_path = self.output_dir.join(JSON_FILE);
        fs::write(&json_path, serde_json::to_string_pretty(&self.training_data)?)
            .await
            .with_context(|| format!("Failed to write training data to {}", json_path.display()))?;

        // Save to CSV file
        self.csv_writer.write_record(&[initial_data, analysis])
            .context("Failed to write record to CSV")?;
        self.csv_writer.flush()
            .context("Failed to flush CSV writer")?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();
    
    if cli.num_examples > MAX_EXAMPLES {
        anyhow::bail!("Number of examples cannot exceed {}", MAX_EXAMPLES);
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(&cli.output_dir)
        .await
        .context("Failed to create output directory")?;

    // Initialize data generator
    let mut generator = DataGenerator::new(cli.output_dir).await?;

    // Create progress bar
    let pb = ProgressBar::new(cli.num_examples as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("##-")
    );

    // Generate examples
    for _ in 0..cli.num_examples {
        pb.set_message("Generating GitHub data...");
        let initial_data = generator.generate_initial_data().await?;
        
        pb.set_message("Analyzing for security risks...");
        let analysis = generator.analyze_data(&initial_data).await?;
        
        pb.set_message("Saving to training data...");
        generator.save_example(initial_data, analysis).await?;
        
        pb.inc(1);
    }

    pb.finish_with_message("Training data generation completed successfully!");
    Ok(())
}

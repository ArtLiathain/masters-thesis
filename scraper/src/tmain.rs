use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize}; // Added Serialize
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Serialize, Debug)] // Added Serialize here
struct Paper {
    title: String,
    doi: String,
    software_repository: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut page = 1;
    let mut all_papers: Vec<Paper> = Vec::new(); // Storage for all results

    println!("Fetching papers...");

    loop {
        let url = format!("https://joss.theoj.org/papers/in/C++.json?page={}", page);

        let response = client
            .get(&url)
            .header(ACCEPT, "application/json")
            .send()
            .await?;

        let papers: Vec<Paper> = response.json().await?;

        if papers.is_empty() {
            println!("\nFinished fetching. Total papers: {}", all_papers.len());
            break;
        }

        println!("Page {}: found {} papers", page, papers.len());

        // Add this page's results to our master list
        all_papers.extend(papers);

        page += 1;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // --- Saving to File ---
    let file_path = "joss_papers.json";
    let mut file = File::create(file_path)?;

    // serialize_pretty makes the JSON readable for humans
    let json_data = serde_json::to_string_pretty(&all_papers)?;
    file.write_all(json_data.as_bytes())?;

    println!("Successfully saved data to {}", file_path);

    Ok(())
}


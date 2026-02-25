use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize}; // Added Serialize
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Serialize, Debug)]
struct Paper {
    title: String,
    doi: String,
    software_repository: String,
}

pub async fn scrape_joss_papers(
    language: String,
    output_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut page = 1;
    let mut all_papers: Vec<Paper> = Vec::new();

    println!("Fetching papers...");

    loop {
        let url = format!(
            "https://joss.theoj.org/papers/in/{}.json?page={}",
            language, page
        );

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
    let mut file = File::create(&output_file)?;

    // serialize_pretty makes the JSON readable for humans
    let json_data = serde_json::to_string_pretty(&all_papers)?;
    file.write_all(json_data.as_bytes())?;

    println!("Successfully saved data to {}", output_file);

    Ok(())
}

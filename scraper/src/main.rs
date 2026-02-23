use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize, Debug)]
struct RepoStats {
    title: String,
    repo_url: String,
    size_kb: u64,
    commit_count: u32,
    contributor_count: u32,
}

// Minimal struct to catch the size from GitHub's main repo API
#[derive(Deserialize)]
struct GitHubRepoResponse {
    size: u64,
}

async fn get_github_metrics(
    client: &reqwest::Client,
    repo_url: &str,
    token: &str,
) -> Result<(u64, u32, u32), Box<dyn std::error::Error>> {
    // 1. Parse "https://github.com/owner/repo" into "owner/repo"
    let path = repo_url
        .trim_end_matches('/')
        .split("github.com/")
        .collect::<Vec<&str>>()[1];

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "Rust-Thesis-Scraper".parse().unwrap());
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());

    // 2. Get Size from the General Repo API
    let repo_info: GitHubRepoResponse = client
        .get(format!("https://api.github.com/repos/{}", path))
        .headers(headers.clone())
        .send()
        .await?
        .json()
        .await?;

    // 3. Get Commit Count (The Pagination Trick)
    // We request 1 commit per page; the 'last' page number in headers = total commits
    let commit_res = client
        .get(format!(
            "https://api.github.com/repos/{}/commits?per_page=1",
            path
        ))
        .headers(headers.clone())
        .send()
        .await?;
    let commits = extract_count_from_header(commit_res.headers());

    // 4. Get Contributor Count (Similar Trick)
    let contrib_res = client
        .get(format!(
            "https://api.github.com/repos/{}/contributors?per_page=1",
            path
        ))
        .headers(headers.clone())
        .send()
        .await?;
    let contributors = extract_count_from_header(contrib_res.headers());

    Ok((repo_info.size, commits, contributors))
}

fn extract_count_from_header(headers: &HeaderMap) -> u32 {
    if let Some(link) = headers.get("link") {
        let link_str = link.to_str().unwrap_or("");
        // Look for: page=X>; rel="last"
        if let Some(pos) = link_str.rfind("page=") {
            let sub = &link_str[pos + 5..];
            if let Some(end) = sub.find('>') {
                return sub[..end].parse().unwrap_or(1);
            }
        }
    }
    1 // If no link header, there is likely only 1 page/item
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Load your previous JSON file
    let file = File::open("joss_papers.json")?;
    let papers: Vec<serde_json::Value> = serde_json::from_reader(file)?;

    let mut extended_stats = Vec::new();

    for p in papers {
        let repo = p["software_repository"].as_str().unwrap_or("");
        if repo.contains("github.com") {
            println!("Processing: {}", repo);
            if let Ok((size, commits, contribs)) =
                get_github_metrics(&client, repo, github_token).await
            {
                extended_stats.push(RepoStats {
                    title: p["title"].as_str().unwrap().to_string(),
                    repo_url: repo.to_string(),
                    size_kb: size,
                    commit_count: commits,
                    contributor_count: contribs,
                });
            }
            // Sleep to respect secondary rate limits
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        }
    }

    let out_file = File::create("thesis_data_v2.json")?;
    serde_json::to_writer_pretty(out_file, &extended_stats)?;
    Ok(())
}

use git2::{Cred, RemoteCallbacks};
use std::{
    env,
    fs::{self, File},
    path::Path,
};

use crate::git_analyzer::GitAnalyzer;
use crate::storage::Neo4jClient;

fn extract_repo_name(url: &str) -> Option<String> {
    url.split("github.com/")
        .nth(1)?
        .split('/')
        .last()
        .map(|s| s.trim_end_matches(".git").to_string())
}

pub async fn analyse_github_repos(
    json_file: String,
    neo4j_uri: String,
    folder_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Neo4jClient::new(&neo4j_uri).await?;
    client.init_schema().await?;

    let file = File::open(json_file)?;
    let repos: Vec<serde_json::Value> = serde_json::from_reader(file)?;
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
            None,
        )
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    let path = Path::new(&folder_path);
    for repo in repos {
        let repo_url = repo["repo_url"].as_str().unwrap_or("");
        if repo_url.contains("github.com") {
            println!("Processing: {}", repo_url);

            fs::remove_dir_all(path).ok();
            builder
                .clone(repo_url, path)
                .map_err(|e| format!("Clone failed: {}", e))?;

            let repo_name = extract_repo_name(repo_url).unwrap_or_else(|| "unknown".to_string());

            let mut analyser = GitAnalyzer::new(path.display().to_string());

            match analyser.analyze(&client, &repo_name).await {
                Ok(commit_count) => {
                    println!(
                        "Saved {} to Neo4j with {} commits analyzed",
                        repo_name, commit_count
                    );
                }
                Err(err) => println!("Error analysing {} repo : {}", repo_url, err),
            }
        }
    }

    Ok(())
}

pub async fn analyze_local_repo(
    repo_path: String,
    repo_name: String,
    neo4j_uri: String,
    prune: bool,
    threshold: i64,
) -> Result<i64, Box<dyn std::error::Error>> {
    let client = Neo4jClient::new(&neo4j_uri).await?;
    client.init_schema().await?;

    let mut analyser = GitAnalyzer::new(repo_path);
    let commit_count = analyser.analyze(&client, &repo_name).await?;

    println!(
        "Saved {} to Neo4j with {} commits analyzed",
        repo_name, commit_count
    );

    if prune {
        let deleted = client.remove_low_importance_connections(threshold).await?;
        println!("Pruned {} edges with weight < {}", deleted, threshold);
    }

    client.compute_hub_scores(&repo_name).await?;
    println!("Computed hub scores for {}", repo_name);

    Ok(commit_count)
}
